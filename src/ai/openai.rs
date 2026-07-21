use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_json::json;
use super::{AiProvider, Completion, Usage};

/// Works for OpenAI, Groq, Mistral, Together, Ollama, LM Studio — anything
/// that speaks the OpenAI chat completions API format.
pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAiProvider {
    pub fn new(api_key: &str, model: &str, base_url: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

#[async_trait::async_trait]
impl AiProvider for OpenAiProvider {
    fn detected_model(&self) -> Option<String> { Some(self.model.clone()) }
    async fn complete(&self, system: &str, user: &str, max_tokens: u32) -> Result<Completion> {
        let body = json!({
            "model": self.model,
            "max_tokens": max_tokens,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user",   "content": user   }
            ]
        });

        let url = format!("{}/v1/chat/completions", self.base_url);

        let res = self.client
            .post(&url)
            .bearer_auth(&self.api_key)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = res.status();
        let json: serde_json::Value = res.json().await?;

        if !status.is_success() {
            let msg = json["error"]["message"].as_str().unwrap_or("Unknown error");
            return Err(anyhow!("API error {}: {}", status, msg));
        }

        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Unexpected response shape from {}", self.base_url))?;

        let u = &json["usage"];
        let input = u["prompt_tokens"].as_u64()
            .or_else(|| u["prompt_eval_count"].as_u64())
            .unwrap_or(0) as u32;
        let output = u["completion_tokens"].as_u64()
            .or_else(|| u["eval_count"].as_u64())
            .unwrap_or(0) as u32;
        if input == 0 && output == 0 {
            Ok(Completion::plain(text))
        } else {
            Ok(Completion::with(text, Usage { input_tokens: input, output_tokens: output, estimated: false }))
        }
    }
}
