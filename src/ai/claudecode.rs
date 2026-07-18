use anyhow::{Result, anyhow};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use super::{AiProvider, Completion, Usage};

pub struct ClaudeCodeProvider {
    binary: String,
}

impl ClaudeCodeProvider {
    pub fn new() -> Self {
        let binary = if std::path::Path::new("/usr/local/bin/claude").exists() {
            "/usr/local/bin/claude".into()
        } else {
            "claude".into()
        };
        Self { binary }
    }
}

#[async_trait::async_trait]
impl AiProvider for ClaudeCodeProvider {
    async fn complete(&self, system: &str, user: &str, max_tokens: u32) -> Result<Completion> {
        let stop = Arc::new(AtomicBool::new(false));
        self.complete_streaming(system, user, max_tokens, Box::new(|_| {}), stop).await
    }

    async fn complete_streaming(
        &self,
        system: &str,
        user: &str,
        _max_tokens: u32,
        on_chunk: Box<dyn Fn(String) + Send + Sync>,
        stop: Arc<AtomicBool>,
    ) -> Result<Completion> {
        let binary = self.binary.clone();
        let system = system.to_string();
        let user   = user.to_string();

        tokio::task::spawn_blocking(move || {
            let mut child = Command::new(&binary)
                .arg("--print")
                .arg("--output-format").arg("stream-json")
                .arg("--include-partial-messages")
                .arg("--verbose")
                .arg("--allowedTools").arg("none")
                .arg("--system-prompt").arg(&system)
                .arg(&user)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()?;

            let stdout = child.stdout.take().unwrap();
            let reader = BufReader::new(stdout);

            let mut full_text = String::new();
            let mut usage: Option<Usage> = None;

            for line in reader.lines() {
                if stop.load(Ordering::Relaxed) {
                    child.kill().ok();
                    child.wait().ok();
                    return Err(anyhow!("__stopped__"));
                }

                let line = match line {
                    Ok(l) => l,
                    Err(_) => break,
                };
                if line.is_empty() { continue; }

                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                    match json["type"].as_str().unwrap_or("") {
                        "stream_event" => {
                            let ev = &json["event"];
                            if ev["type"] == "content_block_delta" {
                                if let Some(chunk) = ev["delta"]["text"].as_str() {
                                    if !chunk.is_empty() {
                                        full_text.push_str(chunk);
                                        on_chunk(chunk.to_string());
                                    }
                                }
                            }
                        }
                        "result" => {
                            // Authoritative final text + usage
                            if let Some(r) = json["result"].as_str() {
                                full_text = r.to_string();
                            }
                            let u = &json["usage"];
                            let input = u["input_tokens"].as_u64().unwrap_or(0) as u32
                                + u["cache_creation_input_tokens"].as_u64().unwrap_or(0) as u32
                                + u["cache_read_input_tokens"].as_u64().unwrap_or(0) as u32;
                            let output = u["output_tokens"].as_u64().unwrap_or(0) as u32;
                            if input > 0 || output > 0 {
                                usage = Some(Usage { input_tokens: input, output_tokens: output, estimated: false });
                            }
                        }
                        _ => {}
                    }
                }
            }

            child.wait().ok();

            if full_text.is_empty() {
                return Err(anyhow!("claude returned empty output — is it logged in?"));
            }
            Ok(match usage {
                Some(u) => Completion::with(full_text, u),
                None    => Completion::plain(full_text),
            })
        })
        .await?
    }
}
