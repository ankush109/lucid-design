use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::{Arc, atomic::AtomicBool};
use super::{AiProvider, Completion, Usage};

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl AiProvider for AnthropicProvider {
    fn detected_model(&self) -> Option<String> { Some(self.model.clone()) }
    async fn complete(&self, system: &str, user: &str, max_tokens: u32) -> Result<Completion> {
        let body = json!({
            "model": self.model,
            "max_tokens": max_tokens,
            "system": system,
            "messages": [{ "role": "user", "content": user }]
        });

        let res = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = res.status();
        let json: serde_json::Value = res.json().await?;

        if !status.is_success() {
            let msg = json["error"]["message"].as_str().unwrap_or("Unknown error");
            return Err(anyhow!("Anthropic API error {}: {}", status, msg));
        }

        let text = json["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Unexpected Anthropic response shape"))?;

        let u = &json["usage"];
        let input = u["input_tokens"].as_u64().unwrap_or(0) as u32
            + u["cache_creation_input_tokens"].as_u64().unwrap_or(0) as u32
            + u["cache_read_input_tokens"].as_u64().unwrap_or(0) as u32;
        let output = u["output_tokens"].as_u64().unwrap_or(0) as u32;
        Ok(Completion::with(text, Usage { input_tokens: input, output_tokens: output, estimated: false }))
    }

    /// Uses Anthropic's prompt caching. The cacheable block is sent with a
    /// `cache_control: ephemeral` marker so identical bytes reused within
    /// 5 minutes cost ~90% less on input.
    async fn complete_streaming_cached(
        &self,
        system: &str,
        cacheable: &str,
        user: &str,
        max_tokens: u32,
        on_chunk: Box<dyn Fn(String) + Send + Sync>,
        _stop: Arc<AtomicBool>,
    ) -> Result<Completion> {
        let mut content: Vec<Value> = Vec::new();
        if !cacheable.trim().is_empty() {
            content.push(json!({
                "type": "text",
                "text": cacheable,
                "cache_control": { "type": "ephemeral" }
            }));
        }
        content.push(json!({ "type": "text", "text": user }));

        let body = json!({
            "model": self.model,
            "max_tokens": max_tokens,
            "system": system,
            "messages": [{ "role": "user", "content": content }]
        });

        let res = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = res.status();
        let j: Value = res.json().await?;

        if !status.is_success() {
            let msg = j["error"]["message"].as_str().unwrap_or("Unknown error");
            return Err(anyhow!("Anthropic API error {}: {}", status, msg));
        }

        let text = j["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Unexpected Anthropic response shape"))?;

        let u = &j["usage"];
        let input_uncached  = u["input_tokens"].as_u64().unwrap_or(0) as u32;
        let cache_creation  = u["cache_creation_input_tokens"].as_u64().unwrap_or(0) as u32;
        let cache_read      = u["cache_read_input_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens   = u["output_tokens"].as_u64().unwrap_or(0) as u32;

        on_chunk(text.clone());
        Ok(Completion::with(
            text,
            Usage {
                input_tokens: input_uncached + cache_creation + cache_read,
                output_tokens,
                estimated: false,
            },
        ))
    }
}
