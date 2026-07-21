use crate::session::Mode;

// ── Mode-scoped design knowledge ──
// The KB is split so LANDING and APP calls each ship only the slice relevant
// to their mode, cutting per-call token cost ~30-50%. Sent as a cacheable
// prefix on every LLM call (Anthropic prompt caching reuses identical bytes
// across calls — the smaller the mode-specific tail, the cheaper).

const KB_CORE:             &str = include_str!("knowledge/core.md");
const KB_PATTERNS_LANDING: &str = include_str!("knowledge/patterns-landing.md");
const KB_PATTERNS_APP:     &str = include_str!("knowledge/patterns-app.md");
const IMAGE_TOOLKIT:       &str = include_str!("knowledge/image-toolkit.md");
const THREEJS_TOOLKIT:     &str = include_str!("knowledge/threejs.md");

pub const SYSTEM_CONTEXT_LANDING: &str = concat!(
    "=== DESIGN KNOWLEDGE (CORE) ===\n",
    include_str!("knowledge/core.md"),
    "\n=== DESIGN KNOWLEDGE (LANDING PATTERNS) ===\n",
    include_str!("knowledge/patterns-landing.md"),
    "\n=== IMAGE TOOLKIT ===\n",
    include_str!("knowledge/image-toolkit.md"),
    "\n=== THREE.JS TOOLKIT ===\n",
    include_str!("knowledge/threejs.md"),
);

pub const SYSTEM_CONTEXT_APP: &str = concat!(
    "=== DESIGN KNOWLEDGE (CORE) ===\n",
    include_str!("knowledge/core.md"),
    "\n=== DESIGN KNOWLEDGE (APP / DASHBOARD PATTERNS) ===\n",
    include_str!("knowledge/patterns-app.md"),
    "\n=== IMAGE TOOLKIT ===\n",
    include_str!("knowledge/image-toolkit.md"),
);

/// Pick the right cacheable prefix for the session's mode. Ambiguous falls
/// back to LANDING as the safe default (superset for undecided sessions).
pub fn system_context(mode: Mode) -> &'static str {
    match mode {
        Mode::App => SYSTEM_CONTEXT_APP,
        Mode::Landing | Mode::Ambiguous => SYSTEM_CONTEXT_LANDING,
    }
}

/// Legacy alias — call sites that pre-date mode routing get the landing
/// context. New code should use `system_context(mode)`.
pub const SYSTEM_CONTEXT: &str = SYSTEM_CONTEXT_LANDING;

// Suppress unused warnings on the split fragment consts — they're kept
// exposed so the MCP surface (slice 5) can serve them independently.
#[allow(dead_code)] fn _kb_touch() {
    let _ = (KB_CORE, KB_PATTERNS_LANDING, KB_PATTERNS_APP, IMAGE_TOOLKIT, THREEJS_TOOLKIT);
}

pub const SKELETON_SYSTEM: &str = "\
You are a senior UI/UX designer. Generate skeleton HTML pages that show \
layout structure without visual styling. Use only neutral colors: white \
background, #333 text, #ddd borders, system fonts. All content must be \
realistic and specific to the product — never use Lorem Ipsum.";

pub const SKELETON_MULTI_SYSTEM: &str = "\
You are a senior UI/UX designer. Produce THREE structurally distinct, \
FULLY STYLED HTML designs for the same product idea, so the user can pick one. \
Each design must:
- Use a genuinely different layout archetype — you MUST match the archetypes \
requested in the user prompt. Diversity is the point.
- Apply the theme direction requested in the user prompt (color palette, mood, \
typography). Every design uses the SAME theme so the user can compare layouts \
without color noise.
- Contain realistic content specific to the product — never Lorem Ipsum.
- Include real image tags using the IMAGE TOOLKIT (hero, feature photos, avatars).
- Be self-contained single-file HTML documents with all CSS in a <style> tag.
- Include hover/focus states on interactive elements.

Output format — output EXACTLY these three blocks, in order, and nothing else:

<!--LAYOUT-1-->
<!--NAME: <2-3 word name>-->
<!DOCTYPE html>…full HTML…
<!--LAYOUT-2-->
<!--NAME: <2-3 word name>-->
<!DOCTYPE html>…full HTML…
<!--LAYOUT-3-->
<!--NAME: <2-3 word name>-->
<!DOCTYPE html>…full HTML…

No markdown. No prose. No explanation. Just the three blocks.";

pub fn skeleton_user(idea: &str, refs: &str, knowledge: &str) -> String {
    format!(r#"Product idea: {idea}

Design references from the web:
{refs}
{knowledge}
Generate a complete single-file HTML skeleton page:
- Include all sections for this product (nav, hero, features, social proof, CTA, footer)
- Each section must have a clear id attribute (id="hero", id="features", id="pricing" etc.)
- Realistic placeholder content specific to this product
- Neutral styling only: white bg, #333 text, #ddd borders, system-ui font
- Layout proportions should be correct — this is a structural wireframe
- Responsive, centered at max-width 1200px
- No decorative colors, no gradients, no shadows

Output ONLY the raw HTML. No markdown. No explanation."#)
}

pub fn skeleton_multi_user(idea: &str, layouts: &str, theme: &str, refs: &str, knowledge: &str) -> String {
    format!(r#"Product idea: {idea}

Layout archetypes for the three variants: {layouts}
Theme direction (color palette, mood, typography): {theme}

Design references from the web:
{refs}
{knowledge}

Generate three FULLY-STYLED HTML designs for "{idea}".

- Variant 1 uses the FIRST layout archetype listed above.
- Variant 2 uses the SECOND.
- Variant 3 uses the THIRD.
If only one archetype was requested, choose two additional complementary archetypes yourself.
If "auto" was requested, pick three genuinely different archetypes from the design knowledge (e.g. bento / editorial / single column).

All three variants share the SAME theme direction — same palette, same typography choices, same signature element. The purpose is to compare LAYOUT strategies with the theme held constant.

Rules per design:
- Include core sections (nav, hero, features/content, social proof, CTA, footer). Use section ids: id="nav", id="hero", id="features", id="pricing" (or similar), id="cta", id="footer".
- Realistic content specific to "{idea}" — never Lorem Ipsum.
- Use image tags with REAL src URLs from the IMAGE TOOLKIT. Choose keywords that match "{idea}".
- Full visual design: color tokens as CSS custom properties, ratio-derived type scale, tinted neutrals, defined hover/focus/active states, one signature element per design.
- Responsive. Mobile is not desktop-minus (reorder for importance).
- Follow the ANTI-TELLS in the knowledge base — avoid cream+serif+terracotta, purple-to-blue gradients, unmotivated glassmorphism unless the theme direction explicitly calls for them.

Follow the exact output format from the system prompt (LAYOUT-N markers and NAME comments)."#)
}

pub const THEME_SYSTEM: &str = "\
You are a senior UI designer and frontend developer. Apply full visual \
design to skeleton HTML pages while preserving every section, heading, \
and content element exactly as written.";

pub fn theme_user(skeleton: &str, tone: &str, knowledge: &str) -> String {
    format!(r#"Here is the skeleton HTML page:
{skeleton}

The user wants this visual tone: "{tone}"
{knowledge}

Apply a complete visual design following the principles above. Specifically:
- Build a NEUTRAL RAMP (9 steps) tinted with 3-8% saturation from the primary hue — never pure gray.
- Pick ONE primary hue matching "{tone}". Optional single accent if the design truly needs a second signal.
- 60/30/10 proportion (neutrals / secondary surface / accent).
- Typography: pick 1-2 Google Fonts fitting the subject. Ratio-derived scale of 6-8 sizes. Real italics available. Load via @import.
- Every button/link needs default, hover, active, and focus states (visible focus ring, 3:1 contrast).
- One SIGNATURE element specific to this subject — not a generic flourish.
- Preserve all image tags, but you may adjust dimensions, add scrims, and add captions. If an image slot is missing, add one from the IMAGE TOOLKIT where it would help.
- All CSS in a single <style> tag using CSS custom properties.
- Preserve every piece of content exactly — change only the styling.
- Keep section id attributes intact (id="hero", id="features", etc.).
- Output one self-contained HTML file, no external CSS files.

AVOID the AI-design tells listed in the knowledge base unless the tone explicitly asks for them:
- Cream + serif + terracotta
- Near-black + acid green/vermilion
- Purple-to-blue gradients
- Unmotivated glassmorphism

Output ONLY the raw HTML. No markdown. No explanation."#)
}

pub const REFINE_SYSTEM: &str = "\
You are a senior UI designer. Refine an existing HTML design based on \
user feedback. Make targeted changes only — preserve everything that \
was not mentioned in the feedback. Follow the spacing scale (4/8/12/16/24/32/48/64), \
the hierarchy law (size > contrast > isolation > position > color), and the \
one-signature-element rule. Do not introduce arbitrary values.";

pub fn refine_user(current: &str, feedback: &str, knowledge: &str) -> String {
    format!(r#"Here is the current HTML design:
{current}

The user wants these changes: "{feedback}"
{knowledge}
Apply the requested changes precisely. Do not restructure or restyle \
sections that were not mentioned. Keep all content and section ids intact. \
Preserve existing image tags. If new images are needed, use the picsum.photos \
or loremflickr.com URL pattern with a descriptive seed.

Output ONLY the updated raw HTML. No markdown. No explanation."#)
}

pub const SKELETON_SINGLE_STYLED_SYSTEM: &str = "\
You are a senior UI/UX designer. Produce ONE fully-styled HTML design for the \
given product idea, using the strongest layout archetype for the subject and \
the requested theme direction.

At the very top of the HTML (before the DOCTYPE), include a single comment \
naming the archetype you chose, exactly in this form:

<meta name=\"archetype\" content=\"bento\">  (or editorial, split-screen, single-column, card-grid, sidebar, z-pattern, magazine, masonry)

CRITICAL — match the design TYPE to the idea:
- Dashboard / analytics tool → sidebar nav, data cards, charts, tables, stat widgets. NOT a landing page.
- Portfolio / personal site → project grid, about section, contact. NOT a landing page.
- Admin panel / internal tool → dense layout, nav rail, data tables, filters. NOT a landing page.
- SaaS marketing site / product launch → hero, features, pricing, CTA. This is the landing page pattern.
- App UI / product screen → actual app chrome: topbar, sidebar, content area with real UI components.
- Blog / editorial → masthead, article grid, typography-first layout.
Read the idea carefully and produce the RIGHT type of interface.

Rules:
- Full visual design: color tokens as CSS custom properties, ratio-derived \
type scale, tinted neutrals, defined hover/focus/active states, one signature \
element specific to the subject.
- Realistic content for the product — never Lorem Ipsum.
- Use image tags with REAL src URLs from the IMAGE TOOLKIT matched to the subject.
- Give top-level sections meaningful ids matching the UI type (e.g. id=\"sidebar\", \
id=\"dashboard\", id=\"stats\" for a dashboard; id=\"projects\", id=\"about\" for a portfolio).
- Nav-item hrefs: for dashboards, admin panels, and app UIs, each sidebar/topbar \
nav item MUST link to a sibling page file, e.g. <a href=\"./workouts.html\">Workouts</a>, \
<a href=\"./calendar.html\">Calendar</a>. Do NOT use href=\"#\" or href=\"#slug\" \
for nav items — those are reserved for same-page scroll anchors on landing pages. \
Use the item label, lowercased and hyphenated, as the filename slug.
- Responsive; mobile is not desktop-minus.
- Follow the ANTI-TELLS in the knowledge base.

Output ONLY the raw HTML. No markdown. No prose. No explanation.";

pub fn skeleton_single_styled_user(
    idea: &str, theme: &str, excluded_archetypes: &str,
    refs: &str, knowledge: &str, initial_pages: &[String],
) -> String {
    let exclusion = if excluded_archetypes.trim().is_empty() {
        String::new()
    } else {
        format!("\n\nARCHETYPES ALREADY TRIED — do NOT reuse any of these: {excluded_archetypes}. Pick a genuinely different structural approach.\n")
    };

    let pages_block = if initial_pages.is_empty() {
        String::new()
    } else {
        let items: Vec<String> = initial_pages.iter().map(|p| {
            let name = p.trim();
            let slug = name.to_lowercase()
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
                .collect::<String>();
            let slug: String = slug.split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-");
            format!("<a href=\"./{slug}.html\">{name}</a>")
        }).collect();
        format!(
            "\n\nMULTI-PAGE APP — the user has already declared these sibling pages. \
Wire them into the sidebar/topbar nav as real anchors so the shell can be \
inherited by later designs. Use these exact hrefs and labels:\n{}\n\
The current (home) item should be marked active (class=\"active\" or aria-current=\"page\").\n",
            items.join("\n")
        )
    };

    format!(r#"Product idea: {idea}

Theme direction: {theme}

Design references from the web:
{refs}
{knowledge}
{exclusion}{pages_block}
Generate ONE complete, fully-styled HTML design for "{idea}".

Reading order first (numbered list of what should be seen first, second, third — internal reasoning, do NOT output this list, use it to drive the layout).

Then output the HTML. Start with the archetype meta tag, then DOCTYPE, then the styled page. Include real image URLs from the IMAGE TOOLKIT matched to "{idea}". Realistic content specific to the subject.
"#)
}

pub const NEXT_PAGES_SUGGEST_SYSTEM: &str = "\
You are a UI information-architect. Given a product idea and the just-designed \
home page's existing section ids, list up to 4 top-level PAGES (not in-page \
sections) this app would plausibly have as siblings of the home page.

Reply as STRICT JSON — an array of objects with `name` (human label, 1-3 words) \
and `slug` (lowercase, hyphenated, filesystem-safe, matches [a-z0-9-]+). Do not \
suggest a page whose slug matches any of the existing section ids you were \
given. Do not suggest generic pages like 'home', 'index', 'landing'.

Output format, exactly:
[{\"name\":\"Workouts\",\"slug\":\"workouts\"},{\"name\":\"Calendar\",\"slug\":\"calendar\"}]

No prose. No markdown fences. No trailing text. Just the JSON array.";

pub fn next_pages_suggest_user(idea: &str, existing_ids: &[String]) -> String {
    let ids = if existing_ids.is_empty() {
        "(none)".to_string()
    } else {
        existing_ids.join(", ")
    };
    format!("Product idea: {idea}\n\nExisting section ids on the home page (do NOT resuggest these as sibling pages): {ids}\n\nList up to 4 sibling pages this app would have. JSON array only.")
}

pub const CRITIQUE_SYSTEM: &str = "\
You are a senior design critic reviewing a completed HTML design against \
strict design principles. Identify AT MOST THREE specific, actionable \
improvements the designer should make.

Priorities in order:
1. Hierarchy — does size / contrast / isolation actually produce the intended reading order? Common failure: three elements all styled as 'important'.
2. Spacing — every dimension from the 4/8/12/16/24/32/48/64 scale; internal group gaps smaller than between-group gaps by ~1:2.
3. Typography — weight jumps skip a step, body 45-75ch, negative letter-spacing on display sizes, real italics used correctly.
4. Contrast — every text-on-background pair meets AA (4.5:1 body, 3:1 large). Especially check tinted backgrounds.
5. Interactive states — visible focus rings at 3:1 contrast, hover/active/disabled defined.
6. One signature element — is there exactly one memorable device, and does it feel drawn from the subject's own world?
7. AI-tells to avoid — cream+serif+terracotta, purple-to-blue gradients, unmotivated glassmorphism, near-black + acid green.

Output STRICTLY a JSON array with 1-3 objects, no markdown fences, no prose, no wrapping:
[
  {\"label\": \"<3-6 word imperative>\", \"prompt\": \"<1-2 sentence refine instruction that, pasted verbatim into a chat, should produce the exact change>\"}
]

Rules:
- Each fix must target ONE specific concrete change. No vague suggestions.
- If the design is truly clean, return 1 fix, not zero.
- Never more than 3.
- No prose before or after the array. Nothing but the array.";

pub fn critique_user(html: &str) -> String {
    format!(r#"HTML design to critique:
{html}

Return 1-3 concrete design improvements as a JSON array (per the system prompt format).
"#)
}

pub const ELEMENT_REFINE_SYSTEM: &str = "\
You are editing a single HTML element inside a larger design. Return ONLY \
the replacement HTML for that one element — same tag, no wrapping, no page \
structure, no other siblings.

Rules:
- Apply visual changes via an inline `style` attribute on the element itself \
so the change is fully scoped and cannot affect siblings.
- Do not modify or add global CSS classes.
- Preserve the element's id and existing classes unless the user explicitly \
asks to change them.
- Preserve the element's inner text and child structure unless the user \
asks otherwise.
- If a text change is requested, preserve any nested structure that the \
user did not ask about.

Output ONLY the raw HTML for that single element. No markdown. No prose.";

pub fn element_refine_user(selector: &str, outer_html: &str, feedback: &str) -> String {
    format!(r#"Selector: {selector}

Current element HTML:
{outer_html}

User request: "{feedback}"

Return the replacement HTML for this element only."#)
}
