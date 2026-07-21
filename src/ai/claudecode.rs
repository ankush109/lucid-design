use anyhow::{Result, anyhow};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use super::{AiProvider, Completion, Usage};

pub struct ClaudeCodeProvider {
    binary: String,
    // Filled the first time we see a `system:init` or `assistant.message`
    // event in the stream — that's when Claude Code discloses its model.
    detected_model: Arc<Mutex<Option<String>>>,
}

impl ClaudeCodeProvider {
    pub fn new() -> Self {
        // Warm the detected-model slot from a persisted cache so the badge
        // shows the real model at startup instead of "…" until the first
        // LLM call. Cache is refreshed after each successful stream.
        let cached = std::fs::read_to_string(Self::cache_path())
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let detected_model = Arc::new(Mutex::new(cached));

        let candidates = [
            "/opt/homebrew/bin/claude",
            "/usr/local/bin/claude",
        ];
        for c in candidates {
            if std::path::Path::new(c).exists() {
                return Self { binary: c.into(), detected_model };
            }
        }
        if let Some(home) = std::env::var_os("HOME") {
            let p = std::path::PathBuf::from(&home).join(".local/bin/claude");
            if p.exists() {
                return Self { binary: p.to_string_lossy().into_owned(), detected_model };
            }
        }
        Self { binary: "claude".into(), detected_model }
    }

    fn cache_path() -> std::path::PathBuf {
        std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
            .join(".config").join("lucid-design").join("claudecode.model")
    }

    fn persist_model(model: &str) {
        let path = Self::cache_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, model);
    }
}

#[async_trait::async_trait]
impl AiProvider for ClaudeCodeProvider {
    async fn complete(&self, system: &str, user: &str, max_tokens: u32) -> Result<Completion> {
        let stop = Arc::new(AtomicBool::new(false));
        self.complete_streaming(system, user, max_tokens, Box::new(|_| {}), stop).await
    }

    fn detected_model(&self) -> Option<String> {
        self.detected_model.lock().ok().and_then(|g| g.clone())
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
        let detected_model = self.detected_model.clone();

        tokio::task::spawn_blocking(move || {
            // Pipe the user prompt via stdin instead of as a CLI arg. The freeform
            // generation path sends 60KB+ of combined SYSTEM_CONTEXT + refs + user
            // prompt — as a positional arg that hangs the shell on macOS. Stdin
            // handles arbitrary-sized input cleanly.
            let mut child = Command::new(&binary)
                .arg("--print")
                .arg("--output-format").arg("stream-json")
                .arg("--include-partial-messages")
                .arg("--verbose")
                .arg("--allowedTools").arg("none")
                .arg("--system-prompt").arg(&system)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            // Write user prompt through stdin — no arg size limits.
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(user.as_bytes()).ok();
                drop(stdin);
            }

            let stderr = child.stderr.take();
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
                    // Sniff the model out of any event that discloses it.
                    // `system:init` fires first with `.model`; assistant
                    // messages carry `.message.model`. Either works.
                    let model_hit = json["model"].as_str()
                        .or_else(|| json["message"]["model"].as_str());
                    if let Some(m) = model_hit {
                        if !m.is_empty() {
                            let mut slot = detected_model.lock().unwrap();
                            if slot.as_deref() != Some(m) {
                                *slot = Some(m.to_string());
                                ClaudeCodeProvider::persist_model(m);
                            }
                        }
                    }

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
                // Surface stderr so the user sees why claude failed instead of a
                // generic "empty output" message (login expired, quota exhausted, etc).
                let err_msg = stderr.and_then(|mut e| {
                    use std::io::Read;
                    let mut s = String::new();
                    e.read_to_string(&mut s).ok().map(|_| s.trim().to_string())
                }).filter(|s| !s.is_empty()).unwrap_or_else(||
                    "claude returned empty output — is it logged in? (run `claude` in your terminal to check)".into()
                );
                return Err(anyhow!("{}", err_msg));
            }
            Ok(match usage {
                Some(u) => Completion::with(full_text, u),
                None    => Completion::plain(full_text),
            })
        })
        .await?
    }
}
