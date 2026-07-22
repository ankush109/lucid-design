// The Refiner specialist.
//
// Takes a rendered HTML design + one CritiqueItem, applies a targeted patch,
// and returns the updated HTML. Uses a smaller-context prompt than a full
// design regen so latency is bounded (~3-6 seconds).

use super::{CritiqueItem, OrchestratorCtx, Specialist};
use crate::ai;
use anyhow::Result;

pub struct Refiner;

#[async_trait::async_trait]
impl Specialist for Refiner {
    fn name(&self) -> &'static str { "refiner" }

    /// `input` is a JSON string: `{"html": "<full doc>", "fix": {…CritiqueItem…}}`.
    /// Returns `{"html": "<patched doc>"}` or bubbles the error.
    async fn run(&self, ctx: &OrchestratorCtx<'_>, input: Option<&str>)
        -> Result<serde_json::Value>
    {
        let raw   = input.ok_or_else(|| anyhow::anyhow!("refiner needs input"))?;
        let payload: serde_json::Value = serde_json::from_str(raw)?;
        let html = payload.get("html").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("refiner input missing html"))?;
        let fix_obj = payload.get("fix")
            .ok_or_else(|| anyhow::anyhow!("refiner input missing fix"))?;
        let fix: CritiqueItem = serde_json::from_value(fix_obj.clone())?;

        let bounded = bound_html(html, 30_000);
        let user = format!(
            "Design HTML:\n{bounded}\n\n\
            Fix to apply — ONE targeted change, keep everything else identical:\n\
            Area: {}\n\
            Change: {}\n\n\
            Return the FULL updated HTML document. No prose, no markdown fences, no partial output.",
            fix.area, fix.prompt
        );

        let stop = ctx.stop.clone();
        let comp = ctx.provider.complete_streaming_cached(
            REFINER_SYSTEM,
            ai::prompts::system_context(ctx.mode),
            &user, 8000,
            Box::new(|_| {}),  // don't stream to the preview iframe
            stop,
        ).await?;
        let cleaned = ai::clean(comp.text);
        // Basic sanity: must still be a full HTML doc, not a fragment.
        if !cleaned.to_ascii_lowercase().contains("<html") {
            return Err(anyhow::anyhow!("refiner output isn't a full HTML document"));
        }
        Ok(serde_json::json!({ "html": cleaned }))
    }
}

fn bound_html(html: &str, max_chars: usize) -> String {
    if html.len() <= max_chars { return html.to_string(); }
    let head = &html[..max_chars * 2 / 3];
    let tail = &html[html.len() - max_chars / 3..];
    format!("{head}\n<!-- middle truncated -->\n{tail}")
}

const REFINER_SYSTEM: &str = "\
You are a senior design engineer applying ONE targeted change to a rendered \
HTML design. The design already exists; you are patching it, not redesigning.\n\n\
Rules:\n\
- Apply the change described in the user's message and nothing else.\n\
- Preserve all other content, structure, palette, fonts, and motion.\n\
- Do not introduce new sections or remove existing ones unless the fix instruction \
explicitly says so.\n\
- Return the FULL updated HTML document — DOCTYPE, head, body, all styles, all \
scripts. Not a diff. Not a fragment. Not markdown.\n\
- If the fix requires changing a CSS value, edit it in the <style> block, don't add \
overriding inline styles unless that's the surgical minimum.\n\
- If the fix is 'reduce three equally-important dominants to one', decide which \
element deserves dominance based on section semantics, then quiet the others via \
reduced weight/size/contrast — don't delete them.\n\n\
Output ONLY raw HTML.";
