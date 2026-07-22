// The orchestrator layer.
//
// Purpose: turn a single-shot LLM generation into a pipeline of focused
// specialists whose outputs are validated before the user ever sees them.
// Each specialist owns one slice of the design problem, has a tight system
// prompt, and returns structured data the orchestrator composes.
//
// v0.6 initial slice ships two specialists: the Critic and the Refiner.
// They wrap the existing single-shot generators (start_design, assemble)
// with an auto-quality-gate — if the critic flags a high-severity issue,
// the refiner patches it silently before the design lands. The user still
// sees the remaining critique items as clickable "polish" chips.
//
// Roadmap slots for future specialists — each is a separate file that
// registers itself via the same `Specialist` trait:
//   - info_architect  (structure decisions: section order, hero archetype)
//   - art_director    (palette + fonts + radii from scraped refs + subject)
//   - copywriter      (headlines, features, testimonials, CTAs)
//   - motion_designer (signature + secondary micro-interactions)
//   - composer        (deterministic Rust that merges everything)
//
// External harnesses (Claude Code, Codex, Cursor, etc.) can reach every
// specialist independently once the MCP surface (slice 5) is wired — the
// `Specialist` trait plus its input/output JSON schemas are the contract.

pub mod critic;
pub mod refiner;

use crate::ai::Provider;
use crate::pipeline::AppEvent;
use crate::session::Mode;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, mpsc::Sender, atomic::AtomicBool};

/// Shared context passed to every specialist. Read-only for now; will grow a
/// mutable outputs map once the multi-specialist pipeline lands.
pub struct OrchestratorCtx<'a> {
    pub brief:    &'a str,
    pub mode:     Mode,
    pub provider: &'a Provider,
    pub tx:       &'a Sender<AppEvent>,
    pub stop:     Arc<AtomicBool>,
}

/// One reviewer note from the Critic. `severity` drives auto-fix decisions:
/// `blocking` → auto-refined before reveal, `high`/`medium` → surfaced as
/// polish chips, `low` → dropped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritiqueItem {
    pub label:    String,
    pub prompt:   String,
    #[serde(default = "default_severity")]
    pub severity: String,
    #[serde(default)]
    pub area:     String,
}

fn default_severity() -> String { "medium".into() }

impl CritiqueItem {
    pub fn is_auto_fix(&self) -> bool {
        matches!(self.severity.as_str(), "blocking" | "high")
    }
}

/// A specialist owns one slice of the design problem. Async because most
/// specialists make an LLM call; a couple (Composer) are pure Rust.
#[async_trait::async_trait]
pub trait Specialist: Send + Sync {
    fn name(&self) -> &'static str;

    /// Run the specialist against `ctx`. Optional `input` (e.g. current HTML
    /// for a critic) is domain-specific.
    async fn run(&self, ctx: &OrchestratorCtx<'_>, input: Option<&str>)
        -> Result<serde_json::Value>;
}

/// Run the auto-quality pass: critic → (if flagged) → refiner → return
/// patched HTML. If anything in the chain fails, return the original HTML
/// unchanged — the auto-loop is best-effort polish, not load-bearing.
///
/// Emits StatusUpdate events so the user sees "Reviewing…" / "Applying quality
/// fix…" instead of a silent latency bump.
pub async fn auto_quality_pass(
    ctx: &OrchestratorCtx<'_>,
    html: String,
    max_fixes: usize,
) -> (String, Vec<CritiqueItem>) {
    let _ = ctx.tx.send(AppEvent::StatusUpdate("Reviewing quality…".into()));
    let critic = critic::Critic;
    let critic_out = match critic.run(ctx, Some(&html)).await {
        Ok(v) => v, Err(_) => return (html, Vec::new()),
    };
    let items: Vec<CritiqueItem> = serde_json::from_value(critic_out)
        .unwrap_or_default();

    // Auto-fix the top N blocking/high-severity items.
    let mut current = html;
    let mut applied  = Vec::new();
    let mut remaining = Vec::new();
    for it in items {
        if it.is_auto_fix() && applied.len() < max_fixes {
            let _ = ctx.tx.send(AppEvent::StatusUpdate(
                format!("Applying quality fix: {}", it.label)
            ));
            let refiner = refiner::Refiner;
            let fix_ctx = serde_json::json!({ "html": current, "fix": &it });
            match refiner.run(ctx, Some(&fix_ctx.to_string())).await {
                Ok(patch) => {
                    if let Some(new_html) = patch.get("html").and_then(|v| v.as_str()) {
                        if !new_html.trim().is_empty() {
                            current = new_html.to_string();
                            applied.push(it);
                            continue;
                        }
                    }
                }
                Err(_) => {} // refiner failure — leave issue as a suggestion
            }
            remaining.push(it);
        } else {
            remaining.push(it);
        }
    }
    (current, remaining)
}
