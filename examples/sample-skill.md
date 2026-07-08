---
name: sample-skill
description: Use when testing skill-compress with a public, non-sensitive SKILL.md sample that contains long runtime rules, optional prose, duplicate business rules, duplicate examples, changelog entries, and protected code blocks.
license: MIT
tags:
  - compression
  - markdown
  - public-fixture
---

# Sample Skill

## Purpose

This public sample is intentionally verbose. It behaves like a realistic Agent Skill while avoiding private names, credentials, customer data, or proprietary procedures. The content is long enough to exercise token estimates, line counts, section stripping, duplicate detection, deterministic cleanup, and LLM-assisted rewrite evaluation.

Use this skill when validating `skill-compress` against a file that has meaningful material to preserve and obvious material to compress. Runtime sections describe repeatable behavior that should survive normal minification. Authoring sections such as changelog, examples, reference notes, and philosophical explanations are intentionally present so feature flags can remove them when requested.

## Changelog

- v0.9.0 - Added a longer routing matrix for intake, editing, verification, and delivery.
- v0.8.0 - Added duplicate business rules to validate reference generation.
- v0.7.0 - Added repeated fenced examples to validate duplicate block replacement.
- v0.6.0 - Added runtime-only scenarios with commands, acceptance checks, and risks.
- v0.5.0 - Added optional prose sections that should be removable by aggressive flags.
- v0.4.0 - Added JSON, Markdown, shell, and TOML code blocks that must keep exact formatting.
- v0.3.0 - Added nonessential implementation notes for compression experiments.
- v0.2.0 - Added public fixture metadata and safety language.
- v0.1.0 - Initial public sample.

---

## Activation

Use this skill when the user asks for help improving, compressing, validating, documenting, or reviewing a Markdown-based agent skill. Use it for repository fixtures, deterministic minification tests, LLM rewrite comparisons, and quality checks around files named `SKILL.md`.

Do not use this skill for secret handling, private customer analysis, regulated advice, legal review, medical review, or direct production deployment. If the request includes credentials or confidential records, stop and ask the user to provide a redacted fixture.

## Inputs

- A source Markdown file that resembles a `SKILL.md` file.
- Optional generated outputs from previous deterministic or LLM compression runs.
- Optional quality constraints such as maximum estimated token count or required sections.
- Optional repository context such as a README, Makefile, CI workflow, or BMAD specification.

## Outputs

- A concise assessment of what changed and why.
- A deterministic minified file when the user requests a concrete rewrite.
- A diff or JSON report when the user requests measurement.
- A list of preserved rules, removed authoring material, and risky semantic changes.

## Core Rules

- Preserve frontmatter keys unless the user explicitly asks to remove them.
- Preserve required runtime instructions even when they appear repetitive.
- Preserve command snippets exactly inside fenced code blocks.
- Preserve JSON field names exactly inside fenced code blocks.
- Preserve TOML keys exactly inside fenced code blocks.
- Preserve shell commands exactly inside fenced code blocks.
- Preserve security warnings when they affect user safety or data handling.
- Preserve acceptance criteria when they define observable behavior.
- Preserve provider names when they define supported integrations.
- Preserve local endpoint guidance when it changes runtime behavior.
- Always update project documentation before committing changes.
- Always update project documentation before committing changes.
- Prefer simple, maintainable, production-ready solutions.
- Prefer simple, maintainable, production-ready solutions.
- Never send source content to an LLM unless the user explicitly enables LLM mode.
- Never send source content to an LLM unless the user explicitly enables LLM mode.
- Reject LLM output that does not look like a complete skill file.
- Reject LLM output that does not look like a complete skill file.
- Keep deterministic compression idempotent across repeated runs.
- Keep deterministic compression idempotent across repeated runs.
- Report compression metrics against the same original baseline.
- Report compression metrics against the same original baseline.
- Treat generated `output/` files as local artifacts unless release docs require them.
- Treat generated `output/` files as local artifacts unless release docs require them.
- Avoid changing commands, flags, model names, or environment variable names during compression.
- Avoid changing commands, flags, model names, or environment variable names during compression.
- Do not infer that two differently worded rules mean the same thing in deterministic mode.
- Do not infer that two differently worded rules mean the same thing in deterministic mode.

## Workflow

1. Read the source file and identify frontmatter, activation text, runtime instructions, examples, references, changelog, and code fences.
2. Preserve frontmatter and runtime behavior before considering size reduction.
3. Normalize obvious Markdown noise such as repeated blank lines, trailing whitespace, decorative headings, and single-line comments.
4. Detect exact duplicate business rules and replace later occurrences with references only when the duplicate is byte-for-byte equivalent after line cleanup.
5. Detect exact duplicate fenced examples and replace later copies with a short reference.
6. Remove optional sections only when a matching feature flag is enabled.
7. Measure the result against the original source and report line, character, word, and estimated token changes.
8. When LLM mode is enabled, compare the LLM rewrite to deterministic output and review for semantic drift.

## Quality Gates

- The file must remain valid Markdown.
- The file must contain frontmatter with `name` and `description`.
- The output must end with exactly one newline.
- A deterministic second pass must produce the same result as the first pass.
- Fenced code block contents must remain intact unless the entire duplicated fence is intentionally replaced by a reference.
- Commands shown in documentation must remain runnable from the repository root.
- Environment variable names must remain stable across README, Makefile, docs, and examples.
- Runtime-only output must still explain when to activate the skill and what constraints govern execution.

## Provider Matrix

| Provider | Mode | Required configuration | Keep during compression |
| --- | --- | --- | --- |
| Anthropic Claude | Remote | `ANTHROPIC_API_KEY` | Yes |
| OpenAI ChatGPT | Remote | `OPENAI_API_KEY` | Yes |
| Mistral | Remote | `MISTRAL_API_KEY` | Yes |
| Google Gemini | Remote | `GEMINI_API_KEY` | Yes |
| Local OpenAI-compatible | Local | `SKILL_COMPRESS_LLM_BASE_URL` | Yes |
| LM Studio | Local | local server URL | Yes |
| Ollama | Local | local server URL | Yes |
| llama.cpp server | Local | local server URL | Yes |

## Protected Command Blocks

The following commands are duplicated later on purpose. A compressor may replace the second identical fenced block with a reference, but it must never rewrite the commands inside the first block.

```bash
cargo run -- examples/sample-skill.md
cargo run -- --diff examples/sample-skill.md
cargo run -- --runtime-only --diff examples/sample-skill.md
make sample-all
```

The following JSON object contains stable field names that should be preserved exactly.

```json
{
  "provider": "gemini",
  "model": "gemini-3.5-flash",
  "input": "examples/sample-skill.md",
  "output": "output/sample-skill.gemini.md",
  "write": false
}
```

The following TOML profile mirrors release-oriented settings and must keep key names unchanged.

```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = "symbols"
panic = "abort"
```

## Duplicate Examples

The next command block is intentionally identical to the first protected command block. It exists to validate duplicate fenced block replacement.

```bash
cargo run -- examples/sample-skill.md
cargo run -- --diff examples/sample-skill.md
cargo run -- --runtime-only --diff examples/sample-skill.md
make sample-all
```

The next JSON block is intentionally identical to the earlier JSON block. It exists to validate that repeated examples compress without modifying the original protected block.

```json
{
  "provider": "gemini",
  "model": "gemini-3.5-flash",
  "input": "examples/sample-skill.md",
  "output": "output/sample-skill.gemini.md",
  "write": false
}
```

## Scenario 01 - Intake And Scope

When a request arrives, identify whether the user wants analysis, deterministic compression, LLM rewriting, documentation updates, or release preparation. Keep the answer anchored to the actual repository state and avoid guessing about files that can be inspected locally.

- Preserve frontmatter keys unless the user explicitly asks to remove them.
- Record whether the source file is public, private, generated, or mixed.
- Keep source and output paths visible in reports.
- Use deterministic mode first when the user asks for safety or repeatability.
- Escalate to LLM mode only when the user explicitly asks for semantic rewrite.
- Keep credentials out of examples, reports, diffs, and generated Markdown.
- Verify that the file remains a complete skill after any rewrite.
- Note sections that look useful for authoring but unnecessary at runtime.

Acceptance:
- The final response names the inspected file.
- The final response states whether the output changed.
- The final response mentions verification commands when any were run.

## Scenario 02 - Frontmatter Preservation

Frontmatter is small, but it carries identity, routing, and package metadata. Compression should normalize surrounding whitespace without dropping useful keys. The description may be long enough to trigger a diagnostic, but a diagnostic is not permission to delete it.

- Preserve frontmatter keys unless the user explicitly asks to remove them.
- Preserve `name`, `description`, `license`, and public tags.
- Keep YAML indentation valid and human-readable.
- Avoid converting YAML lists into comma-separated prose.
- Keep provider names and model names unchanged when they appear in metadata.
- Keep descriptions truthful after removing nonessential sections.
- Flag suspicious frontmatter instead of silently repairing unknown fields.
- Do not invent repository URLs, owners, badges, or package names.

Acceptance:
- YAML delimiters remain at the top of the file.
- The skill still has a readable `name` and `description`.
- Unknown keys survive deterministic minification.

## Scenario 03 - Runtime Instructions

Runtime instructions define behavior. They must stay concise but complete. A minifier may remove decorative prose, but it should not weaken constraints that prevent unsafe provider calls, destructive file edits, or unsupported assumptions.

- Preserve required runtime instructions even when they appear repetitive.
- Preserve security warnings when they affect user safety or data handling.
- Preserve acceptance criteria when they define observable behavior.
- Keep deterministic compression idempotent across repeated runs.
- Avoid changing commands, flags, model names, or environment variable names during compression.
- Report compression metrics against the same original baseline.
- Keep exact wording for refusal and safety boundaries when meaning matters.
- Prefer direct rules over narrative when rewriting manually.

Acceptance:
- Runtime-only output still explains when to use the skill.
- Safety rules remain visible after optional prose is stripped.
- The output can be reread without consulting the original.

## Scenario 04 - Documentation Alignment

Documentation should stay synchronized with code behavior. When the CLI adds a flag, provider, sample target, or report field, public docs and BMAD specs should follow. This scenario intentionally repeats documentation rules for duplicate detection.

- Always update project documentation before committing changes.
- Always update project documentation before committing changes.
- Keep README commands aligned with Makefile targets.
- Keep BMAD requirements aligned with implemented feature flags.
- Keep AGENTS guidance aligned with repository structure.
- Keep sample paths public and avoid private fixture names.
- Mention generated outputs as local artifacts unless they are intentionally published.
- Use examples that run from the repository root.

Acceptance:
- The README mentions the same sample path as the Makefile.
- The BMAD docs mention any new compression flag.
- The contributor guide does not reference private local files.

## Scenario 05 - Deterministic Cleanup

Deterministic cleanup should be boring and predictable. It should remove mechanical waste while preserving user meaning. If a transformation requires interpretation, it belongs in LLM mode or manual review, not the default path.

- Keep deterministic compression idempotent across repeated runs.
- Keep deterministic compression idempotent across repeated runs.
- Normalize trailing whitespace outside code fences.
- Collapse repeated blank lines outside code fences.
- Remove single-line HTML comments outside code fences.
- Normalize decorative horizontal rules.
- Remove decorative bold markers from headings.
- Keep fenced code block contents unchanged.
- Avoid changing list numbering when it may carry meaning.

Acceptance:
- Running the tool twice produces the same deterministic output.
- The output ends with exactly one newline.
- Markdown remains readable in plain text.

## Scenario 06 - Business Rule References

Business rule references are only safe when the repeated rule is exact. Similar wording can hide important differences, so deterministic mode should not merge paraphrases. This scenario repeats a few exact rules and includes one near miss.

- Do not infer that two differently worded rules mean the same thing in deterministic mode.
- Do not infer that two differently worded rules mean the same thing in deterministic mode.
- Prefer simple, maintainable, production-ready solutions.
- Prefer simple, maintainable, production-ready solutions.
- Preserve required runtime instructions even when they appear repetitive.
- Preserve required runtime instructions when they appear in multiple sections.
- Assign references only after the first exact repeated rule.
- Keep the first occurrence readable with its generated identifier.
- Use short references for later exact copies.

Acceptance:
- Exact duplicate rules become references.
- Near-duplicate rules remain independent.
- Generated identifiers are stable for the same input order.

## Scenario 07 - Example Sections

Examples are valuable for authors but often optional at runtime. The tool should preserve examples by default and remove them only when `--strip-examples`, `--strip-nonessential`, or `--runtime-only` is explicit.

- Preserve command snippets exactly inside fenced code blocks.
- Preserve JSON field names exactly inside fenced code blocks.
- Preserve shell commands exactly inside fenced code blocks.
- Keep examples in default deterministic output.
- Remove example-oriented sections only when the user enables the matching flag.
- Keep non-example runtime rules even if they mention an example path.
- Preserve duplicate-block references when examples are retained.
- Do not remove an entire runtime scenario just because it contains a command.

Acceptance:
- Default minification keeps this scenario.
- Runtime-only output may remove sections titled `Examples`.
- Protected command text is not rewritten.

## Scenario 08 - Changelog Stripping

Changelog content is useful for maintainers and noisy for runtime agents. The tool should remove it only when the user asks, or keep the latest entries when the user provides a limit.

- Preserve frontmatter keys unless the user explicitly asks to remove them.
- Keep changelog entries in default output.
- Remove changelog entries when `--strip-changelog` is enabled.
- Keep only the latest N list entries when `--keep-latest N` is used.
- Do not treat every list with versions as a changelog.
- Do not remove runtime acceptance lists.
- Keep release notes out of runtime-only output unless required by the user.
- Report compression metrics after changelog stripping.

Acceptance:
- `--strip-changelog --keep-latest 2` keeps two latest entries.
- `--runtime-only` removes the changelog section.
- Runtime rules remain after changelog removal.

## Scenario 09 - Meta Prose

Meta prose explains why a skill exists, how it evolved, and what tradeoffs shaped it. That is useful for maintainers but usually unnecessary for the runtime path. The compressor should recognize common headings without deleting unrelated operational content.

- Preserve security warnings when they affect user safety or data handling.
- Remove sections titled `Reference` only when meta-prose stripping is enabled.
- Remove sections titled `What it is` only when meta-prose stripping is enabled.
- Remove sections titled `Philosophy` only when meta-prose stripping is enabled.
- Keep `Activation`, `Workflow`, `Quality Gates`, and scenario rules.
- Avoid removing provider configuration tables.
- Keep exact command examples unless an entire examples section is stripped.
- Make the stripped output still usable without author notes.

Acceptance:
- Runtime-only output has fewer authoring notes.
- Required behavior survives meta-prose removal.
- The report explains size reduction without claiming semantic equivalence.

## Scenario 10 - LLM Rewrite Review

LLM mode can compress concepts that deterministic mode cannot safely infer. It can also remove nuance, invent capabilities, or change command semantics. Every LLM rewrite must be treated as a proposal until reviewed.

- Never send source content to an LLM unless the user explicitly enables LLM mode.
- Never send source content to an LLM unless the user explicitly enables LLM mode.
- Reject LLM output that does not look like a complete skill file.
- Reject LLM output that does not look like a complete skill file.
- Compare LLM output against original and deterministic output separately.
- Review removed sections for lost constraints.
- Review changed commands for broken workflows.
- Review provider names and model names for hallucinated replacements.
- Prefer smaller LLM batches for very large skills.

Acceptance:
- LLM gain versus original is reported separately.
- LLM gain versus deterministic is reported separately.
- Review notes call out risky semantic changes.

## Scenario 11 - Local Providers

Local providers can improve privacy, cost control, and iteration speed, but they are still external services from the CLI perspective. The tool should not assume that a local endpoint is safe, available, or compatible until configured.

- Preserve local endpoint guidance when it changes runtime behavior.
- Preserve provider names when they define supported integrations.
- Keep `SKILL_COMPRESS_LLM_BASE_URL` unchanged.
- Keep `SKILL_COMPRESS_LLM_API_KEY` unchanged.
- Allow LM Studio, Ollama, and llama.cpp through OpenAI-compatible endpoints.
- Report connection errors without hiding the provider name.
- Avoid logging full prompts when they may contain private skill text.
- Keep deterministic mode independent from local server availability.

Acceptance:
- Deterministic mode works with no server running.
- Local LLM mode requires explicit provider selection or environment configuration.
- Error messages are actionable without exposing source content.

## Scenario 12 - Remote Providers

Remote providers require credentials and explicit user intent. The sample keeps provider configuration visible so tests can confirm that compression does not alter key names, supported vendors, or model examples.

- Preserve provider names when they define supported integrations.
- Keep `ANTHROPIC_API_KEY` unchanged.
- Keep `OPENAI_API_KEY` unchanged.
- Keep `MISTRAL_API_KEY` unchanged.
- Keep `GEMINI_API_KEY` unchanged.
- Do not print API keys in reports, diffs, errors, or docs.
- Prefer environment variables over committed configuration files.
- Allow model override through CLI flags or environment variables.

Acceptance:
- Provider names remain stable after minification.
- API key variable names remain searchable.
- No fake secret values appear in the sample.

## Scenario 13 - Reports

Reports should help users decide whether compression improved a skill. They should separate deterministic gains, runtime-only gains, and LLM gains. They should also be boring enough for CI and precise enough for release notes.

- Report compression metrics against the same original baseline.
- Report compression metrics against the same original baseline.
- Include lines, characters, words, and estimated tokens.
- Include diagnostics without mixing them into rewrite output.
- Keep JSON report field names stable.
- Return nonzero status for failed `--check`.
- Avoid overstating estimated token precision.
- Make human reports readable in a terminal.
- Make JSON reports parseable without ANSI color.

Acceptance:
- Human report names the path, status, and metrics.
- JSON report can be consumed by a script.
- Estimated tokens are described as estimates.

## Scenario 14 - Diff Review

Diffs are the safest way to inspect transformation behavior. They should show exactly what changed and should not hide aggressive removal behind a summary. The user should be able to audit runtime-only and LLM outputs before writing files.

- Preserve command snippets exactly inside fenced code blocks.
- Avoid changing commands, flags, model names, or environment variable names during compression.
- Print unified diffs for deterministic and LLM modes when requested.
- Do not write files when `--diff` is used without `--write`.
- Use stable diff headers for before and after content.
- Keep removed optional sections visible in diff output.
- Prefer small local samples for fast CI.
- Use large samples for stress testing compression ratios.

Acceptance:
- The diff shows deleted changelog entries when stripping is enabled.
- The diff shows duplicate block replacement.
- The diff can be saved as an artifact for manual review.

## Scenario 15 - Write Mode

Write mode changes files and therefore needs conservative behavior. Deterministic write mode can update files directly. LLM write mode must verify that the generated content still resembles a complete skill before writing.

- Reject LLM output that does not look like a complete skill file.
- Preserve frontmatter keys unless the user explicitly asks to remove them.
- Do not write partial LLM completions.
- Do not write provider error messages into the skill file.
- Print whether the file was updated or already clean.
- Keep generated outputs separate from source fixtures when running samples.
- Avoid overwriting private fixtures during public sample generation.
- Keep local generated files ignored by default.

Acceptance:
- `--write` reports `updated` when content changes.
- `--write` reports `already clean` when no change is needed.
- LLM write mode refuses incomplete Markdown.

## Scenario 16 - CI Behavior

Open-source repositories need predictable checks. CI should be able to run without network provider calls, private data, or local servers. LLM integration tests should use fakes or recorded fixtures.

- Never send source content to an LLM unless the user explicitly enables LLM mode.
- Treat generated `output/` files as local artifacts unless release docs require them.
- Run formatting checks in CI.
- Run clippy with warnings denied in CI.
- Run unit tests in CI.
- Avoid real provider calls in default CI.
- Keep samples public and deterministic.
- Use `cargo package --list` to inspect publish contents before release.

Acceptance:
- CI does not require API keys.
- CI does not require files from `input/`.
- CI passes from a fresh clone.

## Scenario 17 - Release Preparation

Release preparation should check metadata, license, docs, package contents, and binary build behavior. Compression features should be documented before a release tag is created.

- Always update project documentation before committing changes.
- Keep README commands aligned with Makefile targets.
- Keep Cargo metadata complete for publication.
- Keep license information visible in package metadata.
- Exclude ignored private fixtures from the package.
- Include public examples and BMAD docs when they help contributors.
- Verify release build flags before publishing.
- Run package verification before tagging.

Acceptance:
- Package list excludes private inputs.
- Release build succeeds.
- Changelog has a current entry.

## Scenario 18 - Security Review

Security review focuses on data flow, credentials, write operations, provider calls, and logs. A compressor should make it harder to leak data, not easier. Short output is not valuable if it hides important safety constraints.

- Preserve security warnings when they affect user safety or data handling.
- Never send source content to an LLM unless the user explicitly enables LLM mode.
- Do not print API keys in reports, diffs, errors, or docs.
- Keep local endpoints treated as external services unless explicitly trusted.
- Redact credentials before provider calls when possible.
- Refuse to commit generated files containing private source text.
- Keep `.env` files ignored.
- Document required environment variables without fake secrets.

Acceptance:
- Security guidance remains after normal minification.
- No secret-like placeholder is included in this public sample.
- LLM mode remains opt-in.

## Scenario 19 - Public Fixture Design

A public fixture should be representative without being real customer material. It should contain enough structure to exercise the tool, and enough harmless redundancy to make compression visible in examples.

- Keep sample paths public and avoid private fixture names.
- Treat generated `output/` files as local artifacts unless release docs require them.
- Use neutral domain examples instead of customer-specific details.
- Use repeated operational rules to test duplicate detection.
- Use repeated fenced blocks to test block deduplication.
- Use optional authoring sections to test stripping flags.
- Keep the sample longer than normal so reports have meaningful metrics.
- Avoid realistic secrets, domains, tickets, invoices, or personal data.

Acceptance:
- The sample can be committed to an open-source repository.
- The sample produces a visible diff under default minification.
- The sample produces a larger diff under runtime-only mode.

## Scenario 20 - Markdown Robustness

Markdown robustness matters because skill files often mix prose, lists, tables, and code blocks. A minifier should avoid rewriting content in ways that break rendering or change nested list meaning.

- Preserve command snippets exactly inside fenced code blocks.
- Preserve JSON field names exactly inside fenced code blocks.
- Preserve TOML keys exactly inside fenced code blocks.
- Keep pipe tables syntactically valid.
- Keep ordered workflow steps readable.
- Keep nested explanatory lines attached to their parent list item.
- Avoid deleting blank lines that separate headings from paragraphs when readability would suffer.
- Preserve code fence language labels.

Acceptance:
- Markdown preview remains coherent.
- Tables still have matching separator rows.
- Fenced blocks open and close correctly.

## Scenario 21 - Diagnostics

Diagnostics should flag suspicious files without becoming a noisy style checker. A long description, a missing field, or a very large body can be useful signals. The user should understand what to fix and why.

- Report compression metrics against the same original baseline.
- Include diagnostics without mixing them into rewrite output.
- Warn when a skill appears very long.
- Warn when description text is too long for effective routing.
- Warn when required frontmatter is missing.
- Avoid failing normal analysis for warnings.
- Make `--check` strict when output would change.
- Keep diagnostics stable enough for tests.

Acceptance:
- Long sample files produce understandable metrics.
- Warnings do not modify content by themselves.
- Check mode fails only for defined failure conditions.

## Scenario 22 - Human Review

Human review is still required for semantic compression. Deterministic output can be trusted for mechanical cleanup, but LLM output needs a reviewer who understands the skill purpose and repository behavior.

- Compare LLM output against original and deterministic output separately.
- Review removed sections for lost constraints.
- Review changed commands for broken workflows.
- Review provider names and model names for hallucinated replacements.
- Ask whether aggressive removal is acceptable when the file doubles as documentation.
- Keep risky changes in review notes.
- Prefer explicit confirmation before writing LLM output.
- Preserve safety constraints even when they look verbose.

Acceptance:
- The review identifies any changed command.
- The review identifies removed provider guidance.
- The review separates size gain from quality gain.

## Scenario 23 - Makefile Samples

Makefile samples should exercise common CLI flows without requiring secrets. Targets that call remote providers must read API keys from the environment and should not run as part of default verification.

- Keep README commands aligned with Makefile targets.
- Use `examples/sample-skill.md` for public deterministic samples.
- Write generated sample outputs into `output/`.
- Keep `sample-all` deterministic unless explicitly documented otherwise.
- Keep `sample-llm-gemini` separate from default verification.
- Use `GEMINI_API_KEY` from the environment for Gemini examples.
- Allow `GEMINI_MODEL` override without editing the Makefile.
- Avoid committing generated sample outputs by default.

Acceptance:
- `make sample` writes a deterministic minified file.
- `make sample-runtime` writes a runtime-only file.
- `make sample-llm-gemini` does not run without user-provided credentials.

## Scenario 24 - Packaging

Packaging should include source, docs, public examples, and contributor guidance. It should exclude private inputs, generated outputs, caches, target directories, and local editor state.

- Exclude ignored private fixtures from the package.
- Include public examples and BMAD docs when they help contributors.
- Include license, README, changelog, security policy, and contributor guide.
- Keep package contents visible before publishing.
- Avoid relying on ignored files for tests.
- Avoid absolute local paths in package metadata.
- Keep repository and homepage metadata public.
- Run package verification before release.

Acceptance:
- `cargo package --list` contains public files only.
- Package verification builds the crate.
- Package metadata does not reference local private paths.

## Scenario 25 - Regression Tests

Regression tests should cover the transformations that are easy to break. The sample file is large, but unit tests should remain focused and fast. Use fixtures for integration behavior when command-level behavior needs coverage.

- Keep deterministic compression idempotent across repeated runs.
- Preserve command snippets exactly inside fenced code blocks.
- Preserve JSON field names exactly inside fenced code blocks.
- Add unit tests near the module under test.
- Add integration tests for CLI output when behavior stabilizes.
- Avoid live LLM calls in normal tests.
- Use fake clients for provider error handling.
- Test runtime-only stripping with representative headings.

Acceptance:
- Unit tests run without network access.
- Tests cover duplicate business rules.
- Tests cover duplicate fenced blocks.

## Scenario 26 - Error Handling

Errors should be clear enough for users and stable enough for automation. Provider errors, parse errors, file errors, and check failures should not be conflated. Exit codes should communicate the class of failure.

- Report connection errors without hiding the provider name.
- Do not write provider error messages into the skill file.
- Return nonzero status for failed `--check`.
- Return nonzero status for unreadable files.
- Return nonzero status for invalid LLM configuration.
- Keep human-readable error messages short.
- Avoid printing full source content in errors.
- Preserve enough context to debug missing environment variables.

Acceptance:
- Missing file errors name the path from the OS where appropriate.
- Missing API key errors name the expected variable.
- Failed checks are distinguishable from provider failures.

## Scenario 27 - Token Budgeting

Token budgeting is approximate and provider-dependent. The tool should present estimates as estimates and avoid pretending to know exact tokenization for every model. Compression evaluation should still be useful.

- Report compression metrics against the same original baseline.
- Avoid overstating estimated token precision.
- Compare deterministic output against original.
- Compare runtime-only output against original.
- Compare LLM output against original.
- Compare LLM output against deterministic output separately.
- Do not add percentage gains together.
- Preserve enough context to explain why a smaller file is still correct.

Acceptance:
- The report avoids claiming exact provider token counts.
- The documentation says gains are not additive.
- The sample is large enough to make ratios meaningful.

## Scenario 28 - Formatting

Formatting choices should make the skill readable to humans and efficient for agents. Compression should remove noise, not create a dense wall of text. The default output should still be pleasant to review in a pull request.

- Prefer simple, maintainable, production-ready solutions.
- Normalize decorative horizontal rules.
- Remove decorative bold markers from headings.
- Keep headings short and descriptive.
- Keep bullets direct and action-oriented.
- Avoid unexplained acronyms in public samples.
- Keep tables narrow enough for terminal review.
- Keep generated references concise.

Acceptance:
- Default output remains easy to scan.
- Runtime-only output remains structured.
- Diffs are understandable without special tooling.

## Scenario 29 - Nonessential Flags

Nonessential stripping is useful when the file is used only by an agent at runtime. It can be harmful when the same file doubles as documentation. The CLI should make aggressive removal explicit.

- Remove changelog entries when `--strip-changelog` is enabled.
- Remove example-oriented sections only when the user enables the matching flag.
- Remove sections titled `Reference` only when meta-prose stripping is enabled.
- Keep `Activation`, `Workflow`, `Quality Gates`, and scenario rules.
- Make `--strip-nonessential` equivalent to a conservative bundle of optional removals.
- Make `--runtime-only` remove history, examples, and meta prose.
- Avoid removing safety rules under aggressive flags.
- Report changed metrics after stripping.

Acceptance:
- Default minification is conservative.
- Aggressive flags are visible in the command.
- Runtime-only output still has enough instructions to operate.

## Scenario 30 - Final Delivery

Final delivery should be concise and grounded in actual work. The user should know what changed, where it changed, and what was verified. Avoid burying the useful result under process details.

- Mention changed files with clickable paths when useful.
- Mention verification commands and whether they passed.
- Mention limitations when a command could not run.
- Avoid telling the user to copy files that already exist in the workspace.
- Keep the answer proportional to the task.
- Suggest follow-up only when it naturally continues the request.
- Do not claim unstaged changes were committed.
- Keep the tone direct and collaborative.

Acceptance:
- The final response names the sample file.
- The final response includes line and size metrics.
- The final response states whether verification passed.

## What It Is

This section is intentionally meta prose. It describes the fixture itself rather than runtime behavior. It should remain in default output and disappear when `--strip-meta-prose`, `--strip-nonessential`, or `--runtime-only` is requested.

The sample has many repeated rules because duplicate detection should be measurable. It also has near-duplicates because deterministic compression should avoid merging similar but not identical rules. That tension is deliberate and useful for regression testing.

## General Principle

Compression is not the same as quality. A shorter file can be worse if it removes activation boundaries, safety language, provider configuration, or acceptance criteria. A longer file can be better if it captures behavior precisely and prevents unsafe shortcuts.

Use deterministic cleanup for safe mechanical reduction. Use explicit flags for optional section removal. Use LLM rewrite only when semantic compression is worth human review.

## Reference

This reference section is intentionally removable. It repeats the public test goals in narrative form so `--strip-meta-prose` has enough material to remove:

- The fixture should be safe to publish.
- The fixture should be large enough to produce meaningful metrics.
- The fixture should include exact duplicate rules.
- The fixture should include near-duplicate rules.
- The fixture should include duplicate fenced blocks.
- The fixture should include removable changelog content.
- The fixture should include removable authoring prose.
- The fixture should include runtime rules that must remain.

## Examples

The examples below are useful for documentation, but they are not required for runtime behavior. They should remain in default output and disappear under example-stripping flags.

```text
Original tokens: 7200
Deterministic tokens: 6100
Runtime tokens: 4300
LLM tokens: 3900
```

```text
deterministic_gain_vs_original = 1 - deterministic_tokens / original_tokens
runtime_gain_vs_original       = 1 - runtime_tokens / original_tokens
llm_gain_vs_original           = 1 - llm_tokens / original_tokens
llm_gain_vs_deterministic      = 1 - llm_tokens / deterministic_tokens
```

## Before After

Before:

```markdown
- Always update project documentation before committing changes.
- Always update project documentation before committing changes.
```

After:

```markdown
- Always update project documentation before committing changes. (BR-001)
- See BR-001.
```

## Avoid

- Avoid deleting safety warnings just because they are long.
- Avoid rewriting command flags in deterministic mode.
- Avoid treating local LLM endpoints as trusted by default.
- Avoid adding fake credentials to public samples.
- Avoid adding private paths to public fixtures.
- Avoid claiming exact token counts when only estimates are available.

## Prefer

- Prefer small deterministic steps before semantic compression.
- Prefer explicit feature flags for optional removals.
- Prefer public fixtures that are long enough to expose regressions.
- Prefer stable reports that can be compared in CI.
- Prefer human review for every LLM rewrite.
- Prefer preserving behavior over maximizing byte reduction.

## Philosophy

The best compression tool should feel uneventful. It should make obvious cleanup automatic, make risky cleanup explicit, and make semantic cleanup reviewable. It should help maintainers reduce noise without erasing the knowledge that makes a skill reliable.

This sample is intentionally more verbose than a production skill. Its job is to reveal behavior, not to be elegant. If the default output is identical to this source, the compressor is probably too timid. If the runtime output removes safety rules, the compressor is probably too aggressive.
