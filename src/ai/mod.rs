mod anthropic;
mod openai;
mod gemini;
pub mod claudecode;
pub mod prompts;

use anyhow::{Result, bail};
use std::sync::{Arc, atomic::AtomicBool};

#[derive(Debug, Clone, Copy, Default)]
pub struct Usage {
    pub input_tokens:  u32,
    pub output_tokens: u32,
    pub estimated:     bool,
}

pub struct Completion {
    pub text:  String,
    pub usage: Option<Usage>,
}

impl Completion {
    pub fn plain(text: String) -> Self { Self { text, usage: None } }
    pub fn with(text: String, usage: Usage) -> Self { Self { text, usage: Some(usage) } }
}

pub fn estimate_tokens(chars: usize) -> u32 { ((chars + 3) / 4) as u32 }

#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    async fn complete(&self, system: &str, user: &str, max_tokens: u32) -> Result<Completion>;

    async fn complete_streaming(
        &self,
        system: &str,
        user: &str,
        max_tokens: u32,
        on_chunk: Box<dyn Fn(String) + Send + Sync>,
        _stop: Arc<AtomicBool>,
    ) -> Result<Completion> {
        let c = self.complete(system, user, max_tokens).await?;
        on_chunk(c.text.clone());
        Ok(c)
    }

    /// Streaming with an explicit cacheable prefix. Providers that support
    /// prompt caching (Anthropic) mark the prefix with `cache_control` so it
    /// can be reused across calls. Default impl folds the prefix into the user
    /// prompt so OpenAI-style implicit caching still benefits from matching
    /// prefixes and non-caching providers work correctly.
    async fn complete_streaming_cached(
        &self,
        system: &str,
        cacheable: &str,
        user: &str,
        max_tokens: u32,
        on_chunk: Box<dyn Fn(String) + Send + Sync>,
        stop: Arc<AtomicBool>,
    ) -> Result<Completion> {
        let combined = if cacheable.is_empty() {
            user.to_string()
        } else {
            format!("{cacheable}\n\n{user}")
        };
        self.complete_streaming(system, &combined, max_tokens, on_chunk, stop).await
    }
}

pub type Provider = Arc<dyn AiProvider>;

pub fn build_provider(
    provider: &str,
    api_key: &str,
    model: &str,
    base_url: Option<&str>,
) -> Result<Provider> {
    match provider.to_lowercase().as_str() {
        "anthropic" | "claude" => Ok(Arc::new(anthropic::AnthropicProvider::new(api_key, model))),
        "openai" | "gpt" => Ok(Arc::new(openai::OpenAiProvider::new(
            api_key,
            model,
            base_url.unwrap_or("https://api.openai.com"),
        ))),
        "groq"     => Ok(Arc::new(openai::OpenAiProvider::new(api_key, model, "https://api.groq.com/openai"))),
        "mistral"  => Ok(Arc::new(openai::OpenAiProvider::new(api_key, model, "https://api.mistral.ai"))),
        "together" => Ok(Arc::new(openai::OpenAiProvider::new(api_key, model, "https://api.together.xyz"))),
        "ollama"   => Ok(Arc::new(openai::OpenAiProvider::new(
            "ollama",
            model,
            base_url.unwrap_or("http://localhost:11434/v1"),
        ))),
        "gemini"      => Ok(Arc::new(gemini::GeminiProvider::new(api_key, model))),
        "claudecode" | "claude-code" | "local" => {
            Ok(Arc::new(claudecode::ClaudeCodeProvider::new()))
        }
        other => bail!(
            "Unknown provider: '{}'. Supported: anthropic, openai, groq, mistral, together, ollama, gemini, claudecode",
            other
        ),
    }
}

pub fn clean(text: String) -> String {
    let s = text.trim();
    let s = s.strip_prefix("```html").unwrap_or(s);
    let s = s.strip_prefix("```").unwrap_or(s);
    let s = s.strip_suffix("```").unwrap_or(s);
    s.trim().to_string()
}
