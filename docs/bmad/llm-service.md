# LLM Service Design

## Purpose

The LLM service provides optional semantic optimization for `SKILL.md` files. It is not part of the deterministic minifier core.

Primary use cases:

- improve `description` trigger quality;
- summarize verbose sections;
- suggest moving detailed material to `references/`;
- generate a reviewable diff;
- explain why a semantic change is safe or risky.

## Provider Requirements

Supported providers:

- Anthropic Claude;
- OpenAI ChatGPT;
- Mistral;
- Google Gemini;
- local OpenAI-compatible endpoints such as LM Studio, Ollama, or llama.cpp server.

## Abstraction

Provider-neutral request shape:

```rust
pub struct LlmRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub temperature: f32,
    pub max_output_tokens: u32,
    pub response_format: ResponseFormat,
}

pub enum ResponseFormat {
    Text,
    Json,
}

pub struct LlmResponse {
    pub provider: ProviderKind,
    pub model: String,
    pub content: String,
    pub usage: Option<TokenUsage>,
    pub elapsed_ms: u64,
}
```

The current implementation uses synchronous request routing in `src/llm.rs`. A future provider trait can be introduced when the service is split into modules:

```rust
pub trait LlmProvider {
    fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError>;
}
```

## Provider Configuration

Common config:

```toml
[llm]
provider = "anthropic"
model = "claude-sonnet-4-5"
temperature = 0.1
max_output_tokens = 16384
timeout_seconds = 300
redact_secrets = true
```

`max_output_tokens` defaults to 16384 and bounds the `--verify-llm` judge's response. Override it with `--max-output-tokens` (or `SKILL_COMPRESS_LLM_MAX_OUTPUT_TOKENS`). `timeout_seconds` defaults to 300 on slower endpoints; override it with `--timeout-seconds` (or `SKILL_COMPRESS_LLM_TIMEOUT_SECONDS`). A zero timeout is rejected with exit code `4`.

Note: secret redaction is a design requirement but is not yet fully implemented in the current v0.1 code path. Until implemented, users must not run `--verify-llm` on files containing secrets.

Environment variables:

```text
ANTHROPIC_API_KEY
OPENAI_API_KEY
MISTRAL_API_KEY
GEMINI_API_KEY
SKILL_COMPRESS_LLM_API_KEY
SKILL_COMPRESS_LLM_PROVIDER
SKILL_COMPRESS_LLM_MODEL
SKILL_COMPRESS_LLM_BASE_URL
SKILL_COMPRESS_LLM_MAX_OUTPUT_TOKENS
SKILL_COMPRESS_LLM_TIMEOUT_SECONDS
```

Local provider examples:

```toml
[llm]
provider = "local"
model = "llama-local"
base_url = "http://localhost:1234/v1"
api_key = "not-needed"
```

Ollama OpenAI-compatible example:

```toml
[llm]
provider = "local"
model = "llama3.1"
base_url = "http://localhost:11434/v1"
api_key = "ollama"
```

llama.cpp server example:

```toml
[llm]
provider = "local"
model = "local-model"
base_url = "http://localhost:8080/v1"
api_key = "not-needed"
```

## Provider Mapping

### Anthropic

Endpoint shape:

- `POST /v1/messages`
- API key from `ANTHROPIC_API_KEY`
- prompt split into system and user message

Notes:

- strong default for careful rewriting and structured reasoning;
- use low temperature;
- request JSON only when provider/model supports it reliably;
- `content` is an array of blocks; concatenate every `text` block rather than reading `content[0]`;
- treat `stop_reason == "max_tokens"` as a truncation error.

### OpenAI

Endpoint shape:

- `POST /v1/chat/completions` or Responses API in a later phase
- API key from `OPENAI_API_KEY`

Notes:

- initial implementation can use chat completions for compatibility with local endpoints;
- later implementation can add Responses API as a separate transport;
- read `choices[0].message.content`; treat `finish_reason == "length"` as a truncation error. Mistral and Local share this adapter and the same truncation check.
- o-series reasoning models (`o1`/`o3`/`o4-mini`, detected by the `o<digit>` name prefix) require `max_completion_tokens` instead of `max_tokens` and reject a custom `temperature`; the request body switches fields only for provider OpenAI + a reasoning model name, so `gpt-4o`/`gpt-4.1`, Mistral, and Local keep the classic `max_tokens` + `temperature`.

### Mistral

Endpoint shape:

- OpenAI-like chat completions API
- API key from `MISTRAL_API_KEY`

Notes:

- useful as a fast EU-friendly provider option;
- keep request body compatible with documented Mistral chat API.

### Gemini

Endpoint shape:

- Google Generative Language API
- API key from `GEMINI_API_KEY`

Notes:

- requires a different request/response adapter;
- normalize output to `LlmResponse`;
- Gemini may split a single answer across several `candidates[0].content.parts[]`; concatenate every `text` part rather than reading `parts[0]`, otherwise the output is truncated;
- treat `finishReason == "MAX_TOKENS"` as a truncation error;
- sample Makefile workflow defaults to `GEMINI_MODEL=gemini-3.5-flash` and reads `GEMINI_API_KEY`.

### Local

Endpoint shape:

- OpenAI-compatible `/v1/chat/completions`
- base URL configurable

Targets:

- LM Studio;
- Ollama OpenAI-compatible API;
- llama.cpp server.

Notes:

- local does not necessarily mean private if endpoint is remote;
- show target base URL in reports;
- support no API key or dummy API key.

## Prompt Contract

The optimizer should request structured output:

```json
{
  "summary": "short explanation",
  "risk_level": "low|medium|high",
  "proposed_skill_md": "full rewritten file or null",
  "patch": "unified diff or null",
  "recommendations": [
    {
      "kind": "shorten|split_reference|description|warning",
      "message": "text",
      "source_section": "optional heading"
    }
  ]
}
```

Prompt rules:

- preserve YAML frontmatter unless explicitly improving `description`;
- preserve commands, paths, tool names, and code blocks;
- do not invent capabilities;
- do not remove safety constraints;
- favor progressive disclosure over deleting important reference material;
- mark uncertain changes as recommendations, not direct edits.

## Safety Controls

Default controls:

- `--verify-llm` required before any provider call;
- planned: redact secrets before sending content;
- show provider, model, and endpoint in the report;
- the judge is advisory: it never rewrites content or changes the deterministic verdict.

Secret redaction candidates:

- API keys;
- bearer tokens;
- private keys;
- passwords in URLs;
- common cloud credentials.

## Error Handling

Error categories:

- missing API key;
- unsupported provider;
- request timeout;
- HTTP status error;
- malformed provider response;
- truncated provider response (provider signalled a max-token stop); raise `--max-output-tokens` or split the skill;
- invalid optimizer JSON;
- unsafe rewrite rejected by post-validation.

The CLI should return exit code `4` for provider errors.

## Implementation Phases

Phase 1:

- provider-neutral request routing;
- local and OpenAI-compatible implementation;
- Anthropic implementation;
- `--verify-llm` judge over the deterministic residue;
- post-validation.

Phase 2:

- Mistral and Gemini implementations;
- secret redaction before provider calls;
- JSON schema validation;
- provider retry policy;
- model-specific capability flags.

Phase 3:

- LLM evaluation harness;
- A/B comparison of deterministic-only vs LLM-assisted output;
- cache provider responses for repeatable tests.
