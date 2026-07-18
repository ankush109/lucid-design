use anyhow::{Result, bail};
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub base_url: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Env vars take priority over config file
        let provider = std::env::var("DESIGN_GEN_PROVIDER")
            .or_else(|_| read_toml_key("provider"))
            .unwrap_or_else(|_| "anthropic".into());

        let model = std::env::var("DESIGN_GEN_MODEL")
            .or_else(|_| read_toml_key("model"))
            .unwrap_or_else(|_| default_model(&provider));

        let api_key = std::env::var("DESIGN_GEN_API_KEY")
            .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
            .or_else(|_| std::env::var("OPENAI_API_KEY"))
            .or_else(|_| read_toml_key("api_key"))
            .unwrap_or_default();

        let base_url = std::env::var("DESIGN_GEN_BASE_URL")
            .or_else(|_| read_toml_key("base_url"))
            .ok()
            .filter(|s| !s.is_empty());

        if api_key.is_empty() && provider != "ollama" && provider != "claudecode" && provider != "claude-code" && provider != "local" {
            bail!(
                "No API key found.\n\
                 Set DESIGN_GEN_API_KEY, or create a config.toml with api_key = \"...\"\n\
                 Example config.toml:\n\
                 \n\
                   provider = \"anthropic\"\n\
                   model    = \"claude-opus-4-7\"\n\
                   api_key  = \"sk-ant-...\"\n"
            );
        }

        Ok(Config { provider, model, api_key, base_url })
    }
}

fn read_toml_key(key: &str) -> Result<String> {
    let content = fs::read_to_string("config.toml")
        .or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            fs::read_to_string(format!("{}/.config/design-gen/config.toml", home))
        })?;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(key) {
            if let Some(val) = line.splitn(2, '=').nth(1) {
                return Ok(val.trim().trim_matches('"').trim_matches('\'').to_string());
            }
        }
    }
    bail!("key '{}' not found in config.toml", key)
}

fn default_model(provider: &str) -> String {
    match provider {
        "anthropic" | "claude" => "claude-sonnet-4-6".into(),
        "openai" | "gpt"       => "gpt-4o".into(),
        "groq"                 => "llama-3.3-70b-versatile".into(),
        "mistral"              => "mistral-large-latest".into(),
        "gemini"               => "gemini-1.5-pro".into(),
        "ollama"               => "llama3".into(),
        _                      => "gpt-4o".into(),
    }
}
