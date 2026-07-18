use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_json::json;
use super::{AiProvider, Completion, Usage};

pub struct GeminiProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl GeminiProvider {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl AiProvider for GeminiProvider {
    async fn complete(&self, system: &str, user: &str, max_tokens: u32) -> Result<Completion> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let body = json!({
            "system_instruction": { "parts": [{ "text": system }] },
            "contents": [{ "role": "user", "parts": [{ "text": user }] }],
            "generationConfig": { "maxOutputTokens": max_tokens }
        });

        let res = self.client
            .post(&url)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = res.status();
        let json: serde_json::Value = res.json().await?;

        if !status.is_success() {
            let msg = json["error"]["message"].as_str().unwrap_or("Unknown error");
            return Err(anyhow!("Gemini API error {}: {}", status, msg));
        }

        let text = json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Unexpected Gemini response shape"))?;

        let u = &json["usageMetadata"];
        let input  = u["promptTokenCount"].as_u64().unwrap_or(0) as u32;
        let output = u["candidatesTokenCount"].as_u64().unwrap_or(0) as u32;
        if input == 0 && output == 0 {
            Ok(Completion::plain(text))
        } else {
            Ok(Completion::with(text, Usage { input_tokens: input, output_tokens: output, estimated: false }))
        }
    }
}
