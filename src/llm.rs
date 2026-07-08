use std::env;
use std::time::Duration;

use clap::ValueEnum;
use serde::Serialize;
use serde_json::json;

use crate::AppError;

#[derive(Clone, Copy, Debug, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Anthropic,
    Openai,
    Mistral,
    Gemini,
    Local,
}

/// Default ceiling for generated tokens. Large enough to return a full SKILL.md
/// in one response; override with --max-output-tokens for very large skills.
const DEFAULT_MAX_OUTPUT_TOKENS: u64 = 16384;

/// Default per-request HTTP timeout. Generous enough for large skills on slow
/// providers; override with --timeout-seconds or SKILL_COMPRESS_LLM_TIMEOUT_SECONDS.
const DEFAULT_TIMEOUT_SECONDS: u64 = 300;

#[derive(Debug)]
pub struct LlmConfig {
    pub provider: ProviderKind,
    pub model: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
    pub max_output_tokens: u64,
}

impl LlmConfig {
    pub fn from_cli(
        provider: Option<ProviderKind>,
        model: Option<String>,
        base_url: Option<String>,
        max_output_tokens: Option<u64>,
        timeout_seconds: Option<u64>,
    ) -> Result<Self, AppError> {
        let provider = provider
            .or_else(provider_from_env)
            .unwrap_or(ProviderKind::Anthropic);
        let model = model
            .or_else(|| env::var("SKILL_COMPRESS_LLM_MODEL").ok())
            .unwrap_or_else(|| default_model(provider).to_string());
        let base_url = base_url
            .or_else(|| env::var("SKILL_COMPRESS_LLM_BASE_URL").ok())
            .unwrap_or_else(|| default_base_url(provider).to_string());
        let max_output_tokens = max_output_tokens
            .or_else(|| {
                env::var("SKILL_COMPRESS_LLM_MAX_OUTPUT_TOKENS")
                    .ok()
                    .and_then(|value| value.parse().ok())
            })
            .unwrap_or(DEFAULT_MAX_OUTPUT_TOKENS);
        let timeout_seconds = timeout_seconds
            .or_else(|| {
                env::var("SKILL_COMPRESS_LLM_TIMEOUT_SECONDS")
                    .ok()
                    .and_then(|value| value.parse().ok())
            })
            .unwrap_or(DEFAULT_TIMEOUT_SECONDS);

        if timeout_seconds == 0 {
            return Err(AppError::new(
                "timeout-seconds must be greater than zero".to_string(),
                4,
            ));
        }

        let api_key = api_key(provider);

        if provider != ProviderKind::Local && api_key.is_none() {
            return Err(AppError::new(
                format!(
                    "missing API key for {:?}; set {}",
                    provider,
                    api_key_env_name(provider)
                ),
                4,
            ));
        }

        Ok(Self {
            provider,
            model,
            base_url,
            api_key,
            timeout_seconds,
            max_output_tokens,
        })
    }
}

impl PartialEq for ProviderKind {
    fn eq(&self, other: &Self) -> bool {
        *self as u8 == *other as u8
    }
}

/// Dispatch a single system+user completion to the configured provider.
fn complete(
    config: &LlmConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, AppError> {
    match config.provider {
        ProviderKind::Anthropic => call_anthropic(config, system_prompt, user_prompt),
        ProviderKind::Openai | ProviderKind::Mistral | ProviderKind::Local => {
            call_openai_compatible(config, system_prompt, user_prompt)
        }
        ProviderKind::Gemini => call_gemini(config, system_prompt, user_prompt),
    }
}

/// One LLM adjudication of a deterministically-missing atom.
#[derive(Debug, Clone)]
pub struct JudgeVerdict {
    /// Index into the `items` slice passed to `judge_missing`.
    pub index: usize,
    /// `preserved` (paraphrased-equivalent), `weakened`, `lost`, or `unknown`.
    pub verdict: String,
    /// Short supporting quote from the candidate (or empty).
    pub evidence: String,
}

/// Experimental LLM-as-judge layer over the deterministic fidelity gate. Given the
/// candidate document and the rules/sections the gate could not match verbatim,
/// ask the model whether each is semantically preserved, weakened, or lost. The
/// deterministic result stays authoritative; these verdicts are advisory.
pub fn judge_missing(
    config: &LlmConfig,
    candidate: &str,
    items: &[String],
) -> Result<Vec<JudgeVerdict>, AppError> {
    let system_prompt = concat!(
        "You are a strict fidelity judge for compressed Agent Skill SKILL.md files. ",
        "You receive the full CANDIDATE document and a numbered list of RULES or SECTIONS ",
        "from the ORIGINAL that were not found verbatim in the candidate. For each item decide:\n",
        "- \"preserved\": its full meaning is present somewhere in the candidate (paraphrased but ",
        "equivalent, no constraint weakened, no scope changed);\n",
        "- \"weakened\": it is present but vaguer, narrower, broader, or its force/scope changed;\n",
        "- \"lost\": it is not present at all.\n",
        "Be conservative: if unsure whether the constraint is fully equivalent, answer \"weakened\". ",
        "Do not credit a rule as preserved because a *similar* rule exists — differently-worded rules ",
        "may encode distinct requirements. Return ONLY a JSON array, one object per item, in order: ",
        "[{\"index\": <number>, \"verdict\": \"preserved|weakened|lost\", \"evidence\": \"<short candidate quote or empty>\"}]"
    );

    let mut list = String::new();
    for (idx, item) in items.iter().enumerate() {
        list.push_str(&format!("{}. {}\n", idx, item));
    }
    let user_prompt = format!(
        "CANDIDATE document:\n---\n{}\n---\n\nItems not found verbatim ({} total):\n{}",
        candidate,
        items.len(),
        list
    );

    let raw = complete(config, system_prompt, &user_prompt)?;
    parse_verdicts(&raw)
}

/// Parse the judge's JSON array, tolerating surrounding prose or code fences.
fn parse_verdicts(raw: &str) -> Result<Vec<JudgeVerdict>, AppError> {
    let cleaned = strip_markdown_fence(raw);
    let start = cleaned
        .find('[')
        .ok_or_else(|| AppError::new("LLM judge response contained no JSON array", 4))?;
    let end = cleaned
        .rfind(']')
        .ok_or_else(|| AppError::new("LLM judge response JSON array was not closed", 4))?;
    let value: serde_json::Value = serde_json::from_str(&cleaned[start..=end])
        .map_err(|error| AppError::new(format!("failed to parse LLM judge JSON: {}", error), 4))?;
    let array = value
        .as_array()
        .ok_or_else(|| AppError::new("LLM judge response was not a JSON array", 4))?;

    Ok(array
        .iter()
        .map(|item| JudgeVerdict {
            index: item["index"].as_u64().unwrap_or(0) as usize,
            verdict: item["verdict"].as_str().unwrap_or("unknown").to_string(),
            evidence: item["evidence"].as_str().unwrap_or("").to_string(),
        })
        .collect())
}

pub fn strip_markdown_fence(output: &str) -> String {
    let trimmed = output.trim();
    if !trimmed.starts_with("```") {
        return ensure_final_newline(trimmed);
    }

    let mut lines = trimmed.lines();
    let first = lines.next().unwrap_or_default();
    if !first.starts_with("```") {
        return ensure_final_newline(trimmed);
    }

    let mut body: Vec<&str> = lines.collect();
    if body.last().is_some_and(|line| line.trim() == "```") {
        body.pop();
    }
    ensure_final_newline(&body.join("\n"))
}

fn call_anthropic(
    config: &LlmConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, AppError> {
    let body = json!({
        "model": config.model,
        "max_tokens": config.max_output_tokens,
        "temperature": 0.1,
        "system": system_prompt,
        "messages": [
            {"role": "user", "content": user_prompt}
        ]
    });

    let value = post_json(
        &format!("{}/v1/messages", config.base_url.trim_end_matches('/')),
        vec![
            ("x-api-key", config.api_key.as_deref().unwrap_or_default()),
            ("anthropic-version", "2023-06-01"),
            ("content-type", "application/json"),
        ],
        &body,
        config.timeout_seconds,
    )?;

    // Anthropic returns content as an array of blocks; join every text block so
    // multi-block responses are not truncated to the first block.
    let text = join_text_blocks(&value["content"], "text")
        .ok_or_else(|| AppError::new("Anthropic response did not contain content[].text", 4))?;

    if value["stop_reason"].as_str() == Some("max_tokens") {
        return Err(truncated_error("Anthropic", "stop_reason max_tokens"));
    }

    Ok(text)
}

/// OpenAI o-series reasoning models (`o1`, `o3`, `o4-mini`, …) reject the classic
/// `max_tokens` field (they require `max_completion_tokens`) and reject a custom
/// `temperature` (only the default is allowed). Detected by the `o<digit>` name
/// prefix so non-reasoning models (`gpt-4o`, `gpt-4.1`) and other providers keep
/// the classic parameters.
fn is_openai_reasoning_model(model: &str) -> bool {
    let mut chars = model.chars();
    matches!(chars.next(), Some('o')) && matches!(chars.next(), Some(c) if c.is_ascii_digit())
}

/// Build the chat-completions request body, choosing the token-limit field and
/// temperature that the target model accepts.
fn build_openai_body(
    config: &LlmConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> serde_json::Value {
    let mut body = json!({
        "model": config.model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ]
    });

    if config.provider == ProviderKind::Openai && is_openai_reasoning_model(&config.model) {
        // Reasoning models: new token-limit field, and no custom temperature.
        body["max_completion_tokens"] = json!(config.max_output_tokens);
    } else {
        body["temperature"] = json!(0.1);
        body["max_tokens"] = json!(config.max_output_tokens);
    }
    body
}

fn call_openai_compatible(
    config: &LlmConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, AppError> {
    let body = build_openai_body(config, system_prompt, user_prompt);

    // Bind the auth header to a local that outlives the post_json borrow, rather
    // than leaking it for the lifetime of the process.
    let authorization = config.api_key.as_ref().map(|key| format!("Bearer {}", key));
    let mut headers = vec![("content-type", "application/json")];
    if let Some(authorization) = &authorization {
        headers.push(("authorization", authorization.as_str()));
    }

    let value = post_json(
        &format!("{}/chat/completions", config.base_url.trim_end_matches('/')),
        headers,
        &body,
        config.timeout_seconds,
    )?;

    let choice = &value["choices"][0];
    let text = choice["message"]["content"].as_str().ok_or_else(|| {
        AppError::new(
            "OpenAI-compatible response did not contain choices[0].message.content",
            4,
        )
    })?;

    if choice["finish_reason"].as_str() == Some("length") {
        return Err(truncated_error("OpenAI-compatible", "finish_reason length"));
    }

    Ok(text.to_owned())
}

fn call_gemini(
    config: &LlmConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, AppError> {
    let body = json!({
        "generationConfig": {
            "temperature": 0.1,
            "maxOutputTokens": config.max_output_tokens
        },
        "contents": [
            {
                "role": "user",
                "parts": [
                    {"text": format!("{}\n\n{}", system_prompt, user_prompt)}
                ]
            }
        ]
    });

    let api_key = config.api_key.as_deref().unwrap_or_default();
    let value = post_json(
        &format!(
            "{}/models/{}:generateContent?key={}",
            config.base_url.trim_end_matches('/'),
            config.model,
            api_key
        ),
        vec![("content-type", "application/json")],
        &body,
        config.timeout_seconds,
    )?;

    let candidate = &value["candidates"][0];
    // Gemini can split a single answer across multiple `parts`, so reading only
    // `parts[0]` truncates the output. Join every text part instead.
    let text = join_text_blocks(&candidate["content"]["parts"], "text").ok_or_else(|| {
        AppError::new(
            "Gemini response did not contain candidates[0].content.parts[].text",
            4,
        )
    })?;

    if candidate["finishReason"].as_str() == Some("MAX_TOKENS") {
        return Err(truncated_error("Gemini", "finishReason MAX_TOKENS"));
    }

    Ok(text)
}

/// Concatenate the `field` string of every block in a JSON array. Providers that
/// return content as an array of blocks (Anthropic `content`, Gemini `parts`) can
/// split one answer across several blocks; reading only the first truncates it.
fn join_text_blocks(blocks: &serde_json::Value, field: &str) -> Option<String> {
    let blocks = blocks.as_array()?;
    let text: String = blocks
        .iter()
        .filter_map(|block| block[field].as_str())
        .collect();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn truncated_error(provider: &str, reason: &str) -> AppError {
    AppError::new(
        format!(
            "{} response was truncated ({}); raise --max-output-tokens or split the skill into references/",
            provider, reason
        ),
        4,
    )
}

/// Build a ureq agent backed by native-tls. rustls (ureq's default backend)
/// treats a missing TLS `close_notify` as a fatal error even when the HTTP
/// response was fully received; several provider endpoints close that way, so
/// native-tls (Secure Transport / schannel / OpenSSL) is used instead.
fn build_agent() -> Result<ureq::Agent, AppError> {
    let connector = native_tls::TlsConnector::new()
        .map_err(|error| AppError::new(format!("failed to initialize TLS: {}", error), 4))?;
    Ok(ureq::AgentBuilder::new()
        .tls_connector(std::sync::Arc::new(connector))
        .build())
}

/// How many times a single request is attempted before giving up. Provider
/// endpoints intermittently drop the connection mid-response ("Unexpected EOF")
/// or return transient overload statuses; a couple of retries turns those flaky
/// failures into successes without masking real errors.
const MAX_ATTEMPTS: u32 = 3;

fn post_json(
    url: &str,
    headers: Vec<(&str, &str)>,
    body: &serde_json::Value,
    timeout_seconds: u64,
) -> Result<serde_json::Value, AppError> {
    let agent = build_agent()?;
    let payload = body.to_string();

    let mut attempt = 0;
    let response = loop {
        attempt += 1;
        let mut request = agent
            .post(url)
            .timeout(Duration::from_secs(timeout_seconds));
        for (name, value) in &headers {
            request = request.set(name, value);
        }

        match request.send_string(&payload) {
            Ok(response) => break response,
            // A non-2xx status carries the provider's error body, which holds the
            // real root cause (overload, bad model, quota, revoked key). ureq's own
            // Display throws that body away and prints only "<url>: status code NNN",
            // so unpack it ourselves. Retry only transient statuses (429/5xx).
            Err(ureq::Error::Status(code, response)) => {
                if is_retryable_status(code) && attempt < MAX_ATTEMPTS {
                    warn_retry(attempt, &format!("HTTP {}", code));
                    std::thread::sleep(retry_backoff(attempt));
                    continue;
                }
                return Err(http_status_error(code, response));
            }
            // Transport failures (DNS, connect, TLS, dropped connection). A mid-
            // response EOF or reset is usually transient, so retry those kinds.
            Err(ureq::Error::Transport(transport)) => {
                if is_retryable_transport(&transport) && attempt < MAX_ATTEMPTS {
                    warn_retry(attempt, &transport.kind().to_string());
                    std::thread::sleep(retry_backoff(attempt));
                    continue;
                }
                return Err(AppError::new(
                    redact_secrets(&format!("LLM request failed (transport): {}", transport)),
                    4,
                ));
            }
        }
    };

    let text = response
        .into_string()
        .map_err(|error| AppError::new(format!("failed to read LLM response: {}", error), 4))?;

    serde_json::from_str(&text)
        .map_err(|error| AppError::new(format!("failed to parse LLM JSON response: {}", error), 4))
}

/// Transient HTTP statuses worth retrying: rate limits and server-side errors.
/// Client errors (400/401/403/404/422) are permanent — retrying only wastes time.
fn is_retryable_status(code: u16) -> bool {
    matches!(code, 408 | 429 | 500 | 502 | 503 | 504 | 529)
}

/// Transient transport failures worth retrying: a connection that failed to
/// establish, or an I/O error such as a dropped connection / "Unexpected EOF"
/// mid-response. DNS/URL/TLS-config errors are left to fail fast.
fn is_retryable_transport(transport: &ureq::Transport) -> bool {
    matches!(
        transport.kind(),
        ureq::ErrorKind::ConnectionFailed | ureq::ErrorKind::Io
    )
}

/// Fixed exponential backoff (no jitter — std has no RNG): 400ms, then 800ms.
fn retry_backoff(attempt: u32) -> Duration {
    Duration::from_millis(400 * (1u64 << (attempt - 1)))
}

/// Note a retry on stderr so it never contaminates the stdout skill payload.
fn warn_retry(attempt: u32, reason: &str) {
    eprintln!(
        "warning: transient LLM failure ({}); retrying (attempt {}/{})",
        reason, attempt, MAX_ATTEMPTS
    );
}

/// Turn a non-2xx HTTP response into an actionable error. Surfaces the provider's
/// own error message plus a plain-language hint for the status class, so the caller
/// sees *why* the call failed instead of a bare status code.
fn http_status_error(code: u16, response: ureq::Response) -> AppError {
    let body = response.into_string().unwrap_or_default();
    let mut message = format!("LLM request failed: HTTP {}", code);
    if let Some(hint) = status_hint(code) {
        message.push_str(&format!(" — {}", hint));
    }
    if let Some(detail) = extract_provider_error(&body) {
        message.push_str(&format!(": {}", detail));
    } else if !body.trim().is_empty() {
        message.push_str(&format!(": {}", truncate(body.trim(), 500)));
    }
    AppError::new(redact_secrets(&message), 4)
}

/// Plain-language hint for an HTTP status class, pointing at the likely fix.
fn status_hint(code: u16) -> Option<&'static str> {
    Some(match code {
        400 => "bad request (check model name and request parameters)",
        401 => "unauthorized (check the API key)",
        403 => "forbidden (key lacks access to this model or project)",
        404 => "not found (check the model name and base URL)",
        408 => "provider-side request timeout (retry, or raise --timeout-seconds)",
        413 => "payload too large (split the skill into references/ or lower input size)",
        422 => "unprocessable request (check model name and parameters)",
        429 => "rate limited or quota exceeded (slow down or check billing)",
        500 | 502 => "provider internal error (usually transient — retry later)",
        503 => "service unavailable — model overloaded (transient; retry later)",
        529 => "provider overloaded (transient; retry later)",
        _ => return None,
    })
}

/// Extract the human-readable message from a provider error body. Anthropic,
/// OpenAI, Mistral and Gemini all nest it under `error`, either as
/// `error.message` (a string) or as a bare `error` string.
fn extract_provider_error(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let error = &value["error"];
    if let Some(message) = error["message"].as_str() {
        // Gemini also carries a `status` (e.g. UNAVAILABLE); include it when present.
        return Some(match error["status"].as_str() {
            Some(status) if !status.is_empty() => format!("{} [{}]", message, status),
            _ => message.to_string(),
        });
    }
    error.as_str().map(str::to_string)
}

/// Redact API keys that ride in URLs (Google puts `key=...` in the query string)
/// so they never land in an error message, log line, or CI output.
fn redact_secrets(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(idx) = rest.find("key=") {
        let (head, tail) = rest.split_at(idx + "key=".len());
        result.push_str(head);
        let end = tail
            .find(|c: char| c == '&' || c.is_whitespace())
            .unwrap_or(tail.len());
        result.push_str("REDACTED");
        rest = &tail[end..];
    }
    result.push_str(rest);
    result
}

/// Truncate on a UTF-8 boundary, appending an ellipsis when clipped.
fn truncate(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_string();
    }
    let mut end = max;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &text[..end])
}

fn provider_from_env() -> Option<ProviderKind> {
    match env::var("SKILL_COMPRESS_LLM_PROVIDER")
        .ok()?
        .to_lowercase()
        .as_str()
    {
        "anthropic" | "claude" => Some(ProviderKind::Anthropic),
        "openai" | "chatgpt" => Some(ProviderKind::Openai),
        "mistral" => Some(ProviderKind::Mistral),
        "gemini" | "google" => Some(ProviderKind::Gemini),
        "local" | "lmstudio" | "lm-studio" | "ollama" | "llama.cpp" => Some(ProviderKind::Local),
        _ => None,
    }
}

fn api_key(provider: ProviderKind) -> Option<String> {
    let key = match provider {
        ProviderKind::Anthropic => env::var("ANTHROPIC_API_KEY").ok(),
        ProviderKind::Openai => env::var("OPENAI_API_KEY").ok(),
        ProviderKind::Mistral => env::var("MISTRAL_API_KEY").ok(),
        ProviderKind::Gemini => env::var("GEMINI_API_KEY").ok(),
        ProviderKind::Local => env::var("SKILL_COMPRESS_LLM_API_KEY").ok(),
    };
    key.filter(|value| !value.is_empty())
}

fn api_key_env_name(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Anthropic => "ANTHROPIC_API_KEY",
        ProviderKind::Openai => "OPENAI_API_KEY",
        ProviderKind::Mistral => "MISTRAL_API_KEY",
        ProviderKind::Gemini => "GEMINI_API_KEY",
        ProviderKind::Local => "SKILL_COMPRESS_LLM_API_KEY",
    }
}

fn default_model(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Anthropic => "claude-sonnet-4-5",
        ProviderKind::Openai => "gpt-4o-mini",
        ProviderKind::Mistral => "mistral-small-latest",
        ProviderKind::Gemini => "gemini-1.5-flash",
        ProviderKind::Local => "local-model",
    }
}

fn default_base_url(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Anthropic => "https://api.anthropic.com",
        ProviderKind::Openai => "https://api.openai.com/v1",
        ProviderKind::Mistral => "https://api.mistral.ai/v1",
        ProviderKind::Gemini => "https://generativelanguage.googleapis.com/v1beta",
        ProviderKind::Local => "http://localhost:1234/v1",
    }
}

fn ensure_final_newline(value: &str) -> String {
    let mut output = value.to_string();
    output.push('\n');
    output
}

#[cfg(test)]
mod tests {
    use super::{join_text_blocks, strip_markdown_fence, LlmConfig, ProviderKind};
    use serde_json::json;

    #[test]
    fn explicit_timeout_flag_overrides_default() {
        let config = LlmConfig::from_cli(
            Some(ProviderKind::Local),
            Some("m".to_string()),
            Some("http://localhost:1234/v1".to_string()),
            None,
            Some(42),
        )
        .expect("local provider needs no api key");
        assert_eq!(config.timeout_seconds, 42);
    }

    #[test]
    fn zero_timeout_is_rejected() {
        let err = LlmConfig::from_cli(
            Some(ProviderKind::Local),
            Some("m".to_string()),
            Some("http://localhost:1234/v1".to_string()),
            None,
            Some(0),
        )
        .expect_err("zero timeout must fail");
        assert_eq!(err.exit_code, 4);
    }

    #[test]
    fn parses_judge_verdicts_with_surrounding_prose() {
        let raw = "Here are my verdicts:\n```json\n[{\"index\":0,\"verdict\":\"preserved\",\"evidence\":\"line 4\"},{\"index\":1,\"verdict\":\"lost\",\"evidence\":\"\"}]\n```\nDone.";
        let verdicts = super::parse_verdicts(raw).expect("valid json array");
        assert_eq!(verdicts.len(), 2);
        assert_eq!(verdicts[0].verdict, "preserved");
        assert_eq!(verdicts[0].evidence, "line 4");
        assert_eq!(verdicts[1].index, 1);
        assert_eq!(verdicts[1].verdict, "lost");
    }

    #[test]
    fn judge_parse_errors_on_non_array() {
        assert!(super::parse_verdicts("no json here").is_err());
    }

    #[test]
    fn strips_markdown_fence_from_llm_output() {
        let output = "```markdown\n---\nname: test\n---\n```";
        assert_eq!(strip_markdown_fence(output), "---\nname: test\n---\n");
    }

    #[test]
    fn joins_multiple_text_blocks() {
        let parts = json!([{"text": "first "}, {"text": "second"}]);
        assert_eq!(
            join_text_blocks(&parts, "text").as_deref(),
            Some("first second")
        );
    }

    #[test]
    fn text_blocks_empty_when_no_text() {
        assert_eq!(join_text_blocks(&json!([]), "text"), None);
        assert_eq!(join_text_blocks(&json!("nope"), "text"), None);
    }

    #[test]
    fn extracts_gemini_error_with_status() {
        let body = r#"{"error":{"code":503,"message":"This model is currently experiencing high demand.","status":"UNAVAILABLE"}}"#;
        assert_eq!(
            super::extract_provider_error(body).as_deref(),
            Some("This model is currently experiencing high demand. [UNAVAILABLE]")
        );
    }

    #[test]
    fn extracts_openai_style_error_without_status() {
        let body =
            r#"{"error":{"message":"Incorrect API key provided","type":"invalid_request_error"}}"#;
        assert_eq!(
            super::extract_provider_error(body).as_deref(),
            Some("Incorrect API key provided")
        );
    }

    #[test]
    fn extract_provider_error_none_on_non_error_body() {
        assert_eq!(super::extract_provider_error("not json"), None);
        assert_eq!(super::extract_provider_error(r#"{"ok":true}"#), None);
    }

    fn config_for(provider: ProviderKind, model: &str) -> LlmConfig {
        LlmConfig {
            provider,
            model: model.to_string(),
            base_url: "http://x".to_string(),
            api_key: Some("k".to_string()),
            timeout_seconds: 10,
            max_output_tokens: 16384,
        }
    }

    #[test]
    fn detects_openai_reasoning_models() {
        for m in ["o1", "o1-mini", "o3", "o3-mini", "o4-mini"] {
            assert!(super::is_openai_reasoning_model(m), "{} is reasoning", m);
        }
        for m in [
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4.1",
            "openai",
            "mistral-small",
        ] {
            assert!(!super::is_openai_reasoning_model(m), "{} is not", m);
        }
    }

    #[test]
    fn reasoning_body_uses_completion_tokens_and_no_temperature() {
        let body = super::build_openai_body(&config_for(ProviderKind::Openai, "o4-mini"), "s", "u");
        assert_eq!(body["max_completion_tokens"], 16384);
        assert!(body.get("max_tokens").is_none(), "must not send max_tokens");
        assert!(
            body.get("temperature").is_none(),
            "must not send temperature"
        );
    }

    #[test]
    fn classic_body_uses_max_tokens_and_temperature() {
        let body =
            super::build_openai_body(&config_for(ProviderKind::Openai, "gpt-4o-mini"), "s", "u");
        assert_eq!(body["max_tokens"], 16384);
        assert_eq!(body["temperature"], 0.1);
        assert!(body.get("max_completion_tokens").is_none());
    }

    #[test]
    fn non_openai_o_named_model_keeps_classic_params() {
        // An "o1"-looking name on a local/Mistral endpoint must NOT switch fields.
        let body = super::build_openai_body(&config_for(ProviderKind::Local, "o1-local"), "s", "u");
        assert_eq!(body["max_tokens"], 16384);
        assert!(body.get("max_completion_tokens").is_none());
    }

    #[test]
    fn retryable_status_covers_transient_only() {
        for code in [408, 429, 500, 502, 503, 504, 529] {
            assert!(super::is_retryable_status(code), "{} should retry", code);
        }
        for code in [200, 400, 401, 403, 404, 422] {
            assert!(!super::is_retryable_status(code), "{} must not retry", code);
        }
    }

    #[test]
    fn retry_backoff_grows_and_is_bounded() {
        assert_eq!(super::retry_backoff(1).as_millis(), 400);
        assert_eq!(super::retry_backoff(2).as_millis(), 800);
        // Never sleeps past the last attempt's backoff.
        assert!(super::retry_backoff(super::MAX_ATTEMPTS - 1).as_millis() <= 800);
    }

    #[test]
    fn status_hint_flags_overload_and_auth() {
        assert!(super::status_hint(503).unwrap().contains("overloaded"));
        assert!(super::status_hint(401).unwrap().contains("API key"));
        assert_eq!(super::status_hint(418), None);
    }

    #[test]
    fn redacts_api_key_in_query_string() {
        let leaky = "LLM request failed (transport): https://host/v1beta/models/m:generateContent?key=AQ.secret123: connection refused";
        let clean = super::redact_secrets(leaky);
        assert!(!clean.contains("AQ.secret123"));
        assert!(clean.contains("key=REDACTED"));
        // The rest of the message is preserved.
        assert!(clean.contains("connection refused"));
    }

    #[test]
    fn truncate_respects_char_boundary() {
        assert_eq!(super::truncate("hello", 10), "hello");
        assert_eq!(super::truncate("hello world", 5), "hello…");
        // A multi-byte char straddling the cut must not panic.
        let s = "a".repeat(4) + "é";
        assert_eq!(super::truncate(&s, 5), "aaaa…");
    }
}
