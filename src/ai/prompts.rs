/// The full design knowledge base + image toolkit + three.js toolkit. Sent as
/// a cacheable prefix on every LLM call so Anthropic prompt caching can reuse
/// it — the same bytes across calls are what makes caching work at all.
pub const SYSTEM_CONTEXT: &str = concat!(
    "=== DESIGN KNOWLEDGE ===\n",
    include_str!("design_knowledge.md"),
    "\n=== IMAGE TOOLKIT ===\n",
    r#"
Use these free CDN URLs — no API keys required. Choose dimensions that match
the layout slot (hero, card, avatar). Always include descriptive alt text.

1. Topical photos (keyword-tagged, Creative Commons):
   https://loremflickr.com/{WIDTH}/{HEIGHT}/{KEYWORD1},{KEYWORD2}?lock={SEED}
   Example (studio photography hero):
   <img src="https://loremflickr.com/1600/900/studio,photography,camera?lock=42"
        alt="Photographer setting up lights in a studio" />

2. Deterministic random photos (topic-less but reliable):
   https://picsum.photos/seed/{DESCRIPTIVE-SEED}/{WIDTH}/{HEIGHT}
   Example: https://picsum.photos/seed/pricing-hero/1200/600

3. User avatars (1-70, real headshot-style):
   https://i.pravatar.cc/{SIZE}?img={1..70}
   Example: https://i.pravatar.cc/72?img=13

Image placement principles:
- One large hero image, aspect ratio 16:9 or 3:2. Match keyword to the subject.
- Feature cards benefit from a moment/product photo, not just an icon.
- Testimonials use real-looking avatars via pravatar (pick different img= numbers).
- For text over images add a scrim / gradient overlay (contrast floor 4.5:1).
- Consistent aspect ratios across peer elements (all feature cards use 4:3, etc.).
- Loading: reserve the image's aspect ratio in CSS so nothing shifts.
- Never use lorem ipsum captions — write realistic ones tied to the subject.
"#,
    "\n=== THREE.JS TOOLKIT ===\n",
    r#"
When the design calls for a signature 3D moment — a hero with an animated
gradient orb, a floating geometry, a particle field, a wireframe object —
reach for three.js. Guidance below.

CDN (ES module, always latest stable):
<script type="importmap">
{"imports": {"three": "https://cdn.jsdelivr.net/npm/three@0.170.0/build/three.module.js"}}
</script>
<script type="module">
  import * as THREE from "three";
  // ... your scene here ...
</script>

WHEN to use three.js:
- Exactly ONE moment per page. Never two 3D scenes.
- Decorative only — behind the hero, in an aside, as an ambient background.
- If the REFERENCE SITE uses three.js/WebGL (block will call this out), you
  MUST include a matching 3D element in the new design.
- If the subject has depth-suggestive semantics (space, physics, VR, 3D
  modelling tool, immersive audio), consider it even without reference cue.

WHEN NOT:
- Never for navigation, content, or anything a screen reader must access.
- Never in a bento cell — the tiling breaks the visual metaphor.
- Never over key text without an opacity < 0.5 layer between.

MINIMAL BOILERPLATE (adapt, don't copy verbatim):
<div id="scene" style="position:absolute;inset:0;pointer-events:none;z-index:0"></div>
<script type="module">
import * as THREE from "three";
const host = document.getElementById("scene");
const renderer = new THREE.WebGLRenderer({ alpha:true, antialias:true, powerPreference:"high-performance" });
renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
const resize = () => { renderer.setSize(host.clientWidth, host.clientHeight); camera.aspect = host.clientWidth/host.clientHeight; camera.updateProjectionMatrix(); };
host.appendChild(renderer.domElement);
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(50, host.clientWidth/host.clientHeight, 0.1, 100);
camera.position.z = 4;
// ── your geometry / material here ──
window.addEventListener("resize", resize); resize();
const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
const loop = (t) => { /* animate here — but skip transform updates if `reduce` */ renderer.render(scene, camera); requestAnimationFrame(loop); };
requestAnimationFrame(loop);
</script>

COMMON PATTERNS (write one, not all):
1. FLOATING GRADIENT ORB — icosahedron with MeshBasicMaterial + emissive
   glow, slow rotation + subtle scale pulse. Behind hero copy. Colours from
   the design's palette. Compose with a subtle blur backdrop-filter.
2. PARTICLE DRIFT — BufferGeometry with 800-1500 Points, additive blending,
   opacity 0.3-0.5. Wind-like drift on x-axis. Use for atmosphere, not focus.
3. WIREFRAME OBJECT — TorusKnot or Dodecahedron with a wireframe material
   whose line weight roughly matches the design's body stroke. Slow yaw
   rotation. Feels editorial + technical.
4. GRADIENT PLANE — ShaderMaterial on a plane, mixing two palette colours
   with slow noise. Reads as an animated background wash.

PERFORMANCE + RESPONSIBILITY:
- Cap devicePixelRatio at 2. Higher wastes battery.
- Pause `requestAnimationFrame` when `document.hidden` becomes true.
- Honour `prefers-reduced-motion` — hold the still frame, don't remove the visual entirely (the composition still needs it).
- Never depend on user interaction to render — the initial state must look intentional at t=0.
- The scene must survive removal — no critical content lives inside the WebGL canvas.
"#
);

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
- Responsive; mobile is not desktop-minus.
- Follow the ANTI-TELLS in the knowledge base.

Output ONLY the raw HTML. No markdown. No prose. No explanation.";

pub fn skeleton_single_styled_user(
    idea: &str, theme: &str, excluded_archetypes: &str,
    refs: &str, knowledge: &str,
) -> String {
    let exclusion = if excluded_archetypes.trim().is_empty() {
        String::new()
    } else {
        format!("\n\nARCHETYPES ALREADY TRIED — do NOT reuse any of these: {excluded_archetypes}. Pick a genuinely different structural approach.\n")
    };

    format!(r#"Product idea: {idea}

Theme direction: {theme}

Design references from the web:
{refs}
{knowledge}
{exclusion}
Generate ONE complete, fully-styled HTML design for "{idea}".

Reading order first (numbered list of what should be seen first, second, third — internal reasoning, do NOT output this list, use it to drive the layout).

Then output the HTML. Start with the archetype meta tag, then DOCTYPE, then the styled page. Include real image URLs from the IMAGE TOOLKIT matched to "{idea}". Realistic content specific to the subject.
"#)
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
