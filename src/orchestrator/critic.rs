// The Critic specialist.
//
// Takes a rendered HTML design, checks it against the quality-floor rules
// baked into the KB (contrast, hierarchy, spacing rhythm, anti-tells), and
// returns a JSON array of issues with severity + a concrete fix instruction
// the Refiner can act on.

use super::{OrchestratorCtx, Specialist};
use crate::ai;
use anyhow::Result;
use std::sync::{Arc, atomic::AtomicBool};

pub struct Critic;

#[async_trait::async_trait]
impl Specialist for Critic {
    fn name(&self) -> &'static str { "critic" }

    async fn run(&self, ctx: &OrchestratorCtx<'_>, input: Option<&str>)
        -> Result<serde_json::Value>
    {
        let html = input.ok_or_else(|| anyhow::anyhow!("critic needs HTML input"))?;
        // Cap the input so the critic can focus and stay fast.
        let bounded = bound_html(html, 30_000);
        let user   = ai::prompts::critique_user(&bounded);
        let system = CRITIC_SYSTEM;

        // Non-streaming, small output cap. Same cache prefix as the design pass
        // so the cache is warm from the just-completed generation.
        let stop = ctx.stop.clone();
        let comp = ctx.provider.complete_streaming_cached(
            system,
            ai::prompts::system_context(ctx.mode),
            &user, 900,
            Box::new(|_| {}),
            stop,
        ).await?;

        let cleaned = ai::clean(comp.text);
        let items = parse_critic_json(&cleaned);
        Ok(serde_json::to_value(items).unwrap_or(serde_json::Value::Null))
    }
}

fn bound_html(html: &str, max_chars: usize) -> String {
    if html.len() <= max_chars { return html.to_string(); }
    let head = &html[..max_chars * 2 / 3];
    let tail = &html[html.len() - max_chars / 3..];
    format!("{head}\n<!-- middle truncated -->\n{tail}")
}

/// Enhanced critic prompt: adds `severity` + `area` and demands a concrete
/// fix that the Refiner can drop straight into an element-scoped refine call.
/// Backward-compatible with the older critique format — old fields (label,
/// prompt) still populate; new fields default sanely if missing.
const CRITIC_SYSTEM: &str = "\
You are a senior design critic reviewing a completed HTML design against strict \
design principles. Identify AT MOST FIVE specific, actionable improvements the \
designer should make. Each issue must be one concrete change, not a vague nudge.\n\n\
Priorities in order:\n\
1. Contrast — every text-on-background pair must meet WCAG AA (4.5:1 body, 3:1 large). \
Flag anything below.\n\
2. Hierarchy — is there ONE dominant element per section? Three equally-weighted \
'important' elements is a bug.\n\
3. Spacing — every dimension from the 4/8/12/16/24/32/48/64 scale. Group-gap ratio \
approx 1:2.\n\
4. Typography — weight jumps skip a step; body 45-75ch; letter-spacing tuned to size.\n\
5. Interactive states — visible focus rings, defined hover/active/disabled.\n\
6. One signature element — exactly one memorable device, drawn from the subject's \
world.\n\
7. AI-tells to avoid — cream+serif+terracotta, purple-to-blue gradients, \
unmotivated glassmorphism, near-black + acid green, three-icon feature triads, \
'Get started' CTAs, round-number stats.\n\n\
Output STRICTLY a JSON array with 1-5 objects, no markdown fences, no prose, no \
wrapping. Each object has these EXACT keys:\n\
[\n\
  {\n\
    \"severity\": \"blocking\" | \"high\" | \"medium\" | \"low\",\n\
    \"area\":     \"<section id or CSS selector where the issue lives>\",\n\
    \"label\":    \"<3-6 word imperative that shows on the polish chip>\",\n\
    \"prompt\":   \"<1-2 sentence refine instruction that, pasted verbatim into a chat, produces the exact change>\"\n\
  }\n\
]\n\n\
Severity guide:\n\
- blocking: violates AA contrast, or a critical anti-tell that makes the design look \
AI-generated. Auto-fixed before the user sees the design.\n\
- high: hierarchy failure, missing focus rings, three competing dominants. \
Auto-fixed if slot available.\n\
- medium: spacing off-scale, weak copy specificity, one round-number stat. \
Surfaced as a polish chip the user can click.\n\
- low: subjective preferences. Skip.\n\n\
Rules:\n\
- Each fix must target ONE specific concrete change. No vague suggestions.\n\
- If the design is truly clean, return 1 item with severity=medium. Never zero.\n\
- Never more than 5.\n\
- No prose before or after the array. Nothing but the array.";

/// Parse the critic's JSON output tolerantly. Falls back to the older
/// {label, prompt} shape so the /orchestrator can consume both.
fn parse_critic_json(raw: &str) -> Vec<super::CritiqueItem> {
    let cleaned = raw.trim()
        .trim_start_matches("```json").trim_start_matches("```")
        .trim_end_matches("```").trim();
    let start = cleaned.find('[');
    let end   = cleaned.rfind(']');
    if let (Some(a), Some(b)) = (start, end) {
        if b > a {
            let arr = &cleaned[a..=b];
            // Try the rich shape first.
            if let Ok(items) = serde_json::from_str::<Vec<super::CritiqueItem>>(arr) {
                return items;
            }
            // Fallback: older {label, prompt} → wrap with default severity/area.
            if let Ok(vals) = serde_json::from_str::<Vec<serde_json::Value>>(arr) {
                return vals.into_iter().filter_map(|v| {
                    let label  = v.get("label")?.as_str()?.to_string();
                    let prompt = v.get("prompt")?.as_str()?.to_string();
                    Some(super::CritiqueItem {
                        label, prompt,
                        severity: "medium".into(),
                        area: v.get("area").and_then(|x| x.as_str()).unwrap_or("").into(),
                    })
                }).collect();
            }
        }
    }
    Vec::new()
}

/// Small helper so callers outside the orchestrator can still use the critic's
/// output format (e.g. spawn_critique's old callers).
pub fn stop_flag() -> Arc<AtomicBool> { Arc::new(AtomicBool::new(false)) }
