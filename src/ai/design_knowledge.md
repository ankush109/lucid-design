# Design Knowledge Base

A context document for generating interfaces with deliberate visual judgment rather than templated defaults. Read top to bottom once; refer back by section.

---

## 0. The operating stance

Design decisions are answers to questions, not applications of rules. Before any visual choice, three things must be settled:

1. **Subject** — what is this, concretely? Not "a landing page" but "a scheduling tool for freelance photographers."
2. **Audience** — who reads this, and what do they already know?
3. **Job** — what is the single thing this screen must accomplish?

If the brief doesn't pin these down, pin them yourself and state the assumption. Every subsequent choice — palette, type, layout, density — derives from these three. A palette chosen without a subject is decoration. A palette chosen because the subject is a darkroom-adjacent photography tool is design.

**The specificity test:** for any decision you make, ask "would I have made this same choice for a completely different brief?" If yes, it's a default, not a decision. Defaults are acceptable for the 80% of a design that should be quiet — they are not acceptable for the parts that carry identity.

---

## 1. Layout

### 1.1 Layout is a hypothesis about reading order

Layout encodes what should be seen first, second, third. Before choosing a grid, write the reading order as a numbered list. If the layout doesn't produce that order, the layout is wrong regardless of how it looks.

Reading order is driven by, in descending strength:
- **Size** — the largest thing wins, almost unconditionally
- **Contrast** — a bright element on a dark field beats a large dull one
- **Isolation** — whitespace around an element is attention paid to it
- **Position** — top-left in LTR scripts, but weaker than the three above
- **Color** — weakest as a hierarchy tool, strongest as a categorization tool

Common failure: three elements all styled as "important." Nothing is important. Hierarchy requires demotion as much as promotion.

### 1.2 The spatial system

Pick a base unit (commonly 4px or 8px) and derive every dimension from multiples. This is not aesthetic mysticism — it is the difference between spacing that reads as intentional and spacing that reads as arbitrary.

A workable scale:

```
4   — icon-to-label, tight inline gaps
8   — within a component (label to input)
12  — related elements inside a card
16  — between components in a group
24  — between grouped sections
32  — between distinct content blocks
48  — section padding on mobile
64  — section padding on desktop
96  — major section separation
128 — hero breathing room
```

**Proximity law:** elements closer together are perceived as related. The single most common layout bug is uniform spacing — when the gap between a label and its input equals the gap between two unrelated fields, the eye cannot parse the structure. Spacing *inside* a group must always be smaller than spacing *between* groups. Enforce this ratio at roughly 1:2 minimum.

### 1.3 Grid systems

**12-column** — the workhorse. Divides cleanly into 2, 3, 4, 6. Use when content is heterogeneous and you need flexible spans. Gutters typically 16–32px, margins 24px mobile / 64px+ desktop.

**Modular / symmetric grid** — equal cells, content flows in reading order. Good for galleries, product listings, anything where items are peers.

**Asymmetric / broken grid** — deliberately offsets elements from column boundaries. Creates tension and dynamism; requires strong justification and careful execution or it reads as a mistake. Best used with one anchor element aligned to the grid so the eye has a reference point.

**Baseline grid** — vertical rhythm where all text sits on a shared invisible ruler (line-height as the unit). Expensive to maintain, high payoff for text-dense editorial work. Skip for app UI.

### 1.4 Layout archetypes

Each archetype is a set of promises about content. Choose based on what the content actually is.

---

**Bento grid**

Cells of varying sizes tiled into a rectangle without gaps in the overall silhouette — named for Japanese compartmented lunchboxes.

*Use when:* you have 4–9 features/facts of genuinely unequal importance and each can be summarized in a glance-sized unit. Feature overviews, dashboards, capability showcases.

*Structure:* start from a base grid (commonly 4×3 or 6×4), then merge cells to create the hierarchy. One cell should be clearly dominant — typically 2×2 in a 4×3 — and it holds the most important claim. Remaining cells vary between 1×1, 2×1, and 1×2.

```
┌───────────────┬───────┬───────┐
│               │       │       │
│    HERO       │  1×1  │  1×1  │
│    2×2        │       │       │
│               ├───────┴───────┤
│               │     2×1       │
├───────┬───────┼───────────────┤
│  1×1  │  1×1  │     2×1       │
└───────┴───────┴───────────────┘
```

*Rules that make bento work:*
- Uniform gap between all cells (16–24px typical). Inconsistent gaps destroy the effect immediately.
- Uniform corner radius across all cells.
- Each cell needs internal padding at least equal to the gap, usually 1.5–2×.
- Content inside a cell should be top-left or center aligned *consistently* — mixing alignments across cells looks accidental.
- Do not fill every cell with the same content pattern (icon + heading + paragraph, nine times). Vary the content *type*: one cell holds a number, one a small chart, one an image, one a quote. This variation is the entire point.
- Cells should be visually distinguishable from the background — either by fill, border, or elevation, but pick one method and hold it.

*Failure modes:* uniform cells (that's just a card grid), too many cells (past ~9 it becomes noise), no dominant cell (no hierarchy), and cells whose content doesn't justify their size.

*Mobile:* bento collapses to a single column. Order cells by importance, not by their desktop row order. Consider preserving one 2-wide cell to retain some rhythm.

---

**Z-pattern**

Eye travels top-left → top-right → diagonal → bottom-left → bottom-right. Suits sparse pages with a single conversion goal: logo top-left, nav/secondary action top-right, headline and visual on the diagonal, CTA bottom-right.

*Use when:* minimal content, one action. Marketing hero, splash page.

---

**F-pattern**

Reflects how people actually scan text-heavy pages: two horizontal sweeps at the top, then a vertical scan down the left edge. Implication: front-load meaning in headings and the first few words of each line. Left-align text. Don't hide critical information on the right side of long lines.

*Use when:* documentation, articles, search results, any dense reading surface.

---

**Split screen**

Two vertical halves, often contrasting (image/text, before/after, two audiences). Strong, simple, immediately legible. Risk: it's a very familiar shape. Make it interesting through asymmetric weight (60/40 rather than 50/50), by letting one side bleed to the viewport edge, or by having an element cross the seam.

---

**Card grid**

Repeating equal units. Honest and scannable when items are true peers. Deadly boring when items aren't. If your cards have wildly varying content length, either normalize the content or switch to a list.

*Card anatomy:* image or accent zone → primary label → supporting text → action. Keep this order consistent across every card. Cap the supporting text at 2–3 lines with truncation.

---

**Editorial / magazine**

Mixed column widths, pull quotes, images breaking the text column, generous margins, strong typographic hierarchy. Suits long-form content with a point of view. Requires real content — it collapses with placeholder text.

---

**Masonry**

Variable-height items packed into columns. Right for image sets with mixed aspect ratios. Wrong for anything where comparison matters, because unequal heights make comparison hard.

---

**Sidebar + canvas**

Persistent navigation rail beside a working surface. The default for tools and dashboards. Sidebar 240–320px; collapse to icons under ~1100px; drawer on mobile. Keep the canvas the visual focus — the sidebar should recede in contrast, not compete.

---

**Single column**

Everything stacked, constrained to 60–75ch. The most underrated layout. Excellent for onboarding, forms, articles, and anything where you want zero ambiguity about reading order. If you can't justify a more complex layout, use this one.

---

### 1.5 Density

Three registers, chosen by task:

| Register | Row height | Use for |
|---|---|---|
| Comfortable | 48–56px | Consumer apps, marketing, low-frequency tasks |
| Standard | 40–44px | General app UI |
| Compact | 28–32px | Data tables, professional tools, high-frequency expert use |

Density is not a style preference — it's a function of how often the user performs the task and how much they need on screen at once. A trader's terminal and a meditation app should not have the same density, and neither is "better."

### 1.6 Alignment and the invisible edges

Every element should align to something. When an element aligns to nothing, it reads as an error even when the viewer can't articulate why.

- Prefer left alignment for text. Centered text is acceptable for short headings (under ~2 lines) and is bad for paragraphs.
- Ragged-right is correct for the web. Justified text creates rivers without proper hyphenation.
- Optical alignment beats mathematical alignment for punctuation, round shapes, and italic type. A circle needs to be slightly larger than a square to look the same size; a quotation mark should hang outside the text edge.
- Establish 2–3 vertical alignment edges per screen and hold them ruthlessly. More than that and the structure dissolves.

### 1.7 Responsive strategy

Design at the extremes first — the narrowest and widest viewports — then fill in.

Breakpoints as guidance, not gospel:
```
< 640px    single column, stack everything, 16–24px margins
640–1024   two columns where content allows, 24–32px margins
1024–1440  full layout, 48–64px margins
> 1440     cap content width (1200–1440px), let margins grow
```

Never let line length exceed ~75 characters regardless of viewport. Wide screens get more margin, not longer lines.

Mobile is not desktop-minus. Reorder for importance, replace hover affordances with visible ones, size touch targets at 44×44px minimum, and put primary actions within thumb reach (bottom third of the screen).

---

## 2. Color

### 2.1 The mental model

Think in HSL, not hex. Hue is identity, saturation is intensity, lightness is hierarchy. Most palette problems are lightness problems.

Key perceptual facts:
- Equal lightness values across hues do *not* look equally light. Yellow at 50% lightness appears far brighter than blue at 50%. Use perceptual color spaces (OKLCH, LCH) when precision matters.
- Saturated colors advance toward the viewer; desaturated recede. This is a free hierarchy tool.
- Large areas of saturated color are fatiguing. Saturation belongs on small surfaces.
- Pure gray (S=0) looks dead next to any colored content. Tint your neutrals — pull 3–8% saturation from the primary hue into every gray. This single change is the most reliable upgrade to an amateur palette.

### 2.2 Palette architecture

A complete system needs:

**Neutrals — 9 to 12 steps.** This is 80–90% of what appears on screen. Backgrounds, surfaces, borders, text. Build the ramp with even *perceptual* steps, not even numeric ones — the middle needs more steps than the ends because that's where discrimination is hardest.

```
50   backgrounds (lightest)
100  subtle surface fills
200  hairline borders, dividers
300  borders, disabled states
400  placeholder text, disabled text
500  secondary text (min for AA on white at 4.5:1)
600  body text on light
700  headings
800  high-emphasis text
900  maximum contrast
```

**Primary — one hue, 5 to 9 steps.** Actions, links, selection, focus, brand. Restraint here is what makes it read as intentional. Steps let you use light tints as backgrounds and dark shades for text on those tints.

**Accent — optional, one hue.** Only if you have a second category of thing that genuinely needs distinguishing. Two accents means three brand colors means no brand color.

**Semantic — success, warning, error, info.** These must be distinguishable from primary and from each other. Never make your primary green if success is also green. Error red should be the most saturated color in the system — it should hurt slightly.

Semantic colors need at least three steps: a light background tint, a mid border, and a dark text/icon value. A red banner with red text at the same lightness is unreadable.

### 2.3 Combination strategies

**Monochromatic** — one hue, varied lightness and saturation. Sophisticated, cohesive, low-risk. Needs strong typography and layout to avoid flatness. Add interest through texture, contrast range, and one non-color signature element.

**Analogous** — 2–3 adjacent hues (blue / blue-green / green). Harmonious and calm. Pick one as clearly dominant — 60/30/10 — or it turns muddy.

**Complementary** — opposite hues (blue/orange, purple/yellow). Maximum vibrancy and contrast. Never use at equal saturation and equal area — one must dominate heavily, the other appears as accent only. Avoid placing saturated complements directly adjacent; the edge vibrates.

**Split complementary** — one hue plus the two neighbors of its complement. Most of the tension of complementary with less risk. A good default for a brief with no color direction.

**Triadic** — three evenly spaced hues. Energetic, playful. Very hard to control. Requires one dominant hue and two used sparingly.

**Tetradic** — two complementary pairs. Rich but easy to ruin. Usually more colors than a design needs.

**Neutral + one accent** — the most reliable strategy in interface design. A full neutral ramp plus a single saturated hue used only where the eye must go. Nearly impossible to make ugly; requires real skill in type and layout to make memorable.

### 2.4 Proportion

The 60/30/10 rule as a starting heuristic:
- 60% dominant — usually a neutral background
- 30% secondary — surfaces, containers, secondary text
- 10% accent — actions, highlights, brand moments

In interfaces the accent share is often lower, closer to 5%. When everything is highlighted, nothing is. Count your accent-colored elements on a screen; if there are more than five, cut.

### 2.5 Contrast requirements

Non-negotiable floors (WCAG AA):
- Body text: 4.5:1 against its background
- Text ≥18pt or ≥14pt bold: 3:1
- UI components, focus indicators, meaningful icons: 3:1
- AAA (7:1) for sustained reading surfaces

Test the actual pairs you ship, including text on tinted backgrounds, text on images (add a scrim), disabled states, and hover states.

Color must never be the sole carrier of meaning. Pair with icon, text, position, or pattern. Roughly 8% of men have some form of color vision deficiency, and red/green is the most common axis — which is exactly the axis most status systems use.

### 2.6 Dark mode

Not an inversion. Rebuild it.

- Background should be dark gray, not pure black (#0A0A0B to #18181B typical). Pure black creates harsh contrast and visible smearing on OLED during scroll.
- Elevation is expressed by *lighter* surfaces, not shadows. Shadows barely read on dark.
- Desaturate accents by 10–20% and raise their lightness. A color that sings on white screams on black.
- Pure white text is too harsh — use ~90% lightness.
- Contrast ratios must be re-verified independently. Passing in light mode tells you nothing about dark mode.

### 2.7 Choosing a hue with intent

Hue carries association, and association is culturally variable, but within a Western commercial context:

| Hue | Reads as | Overused in |
|---|---|---|
| Blue | Trust, stability, calm, corporate | SaaS, finance, health — the safe default |
| Green | Growth, nature, money, permission | Fintech, sustainability, wellness |
| Purple | Creative, premium, mystical | Creator tools, AI products |
| Red | Urgency, appetite, danger, passion | Food, entertainment, alerts |
| Orange | Energy, warmth, approachability, play | Consumer marketplaces |
| Yellow | Optimism, attention, caution | Rare as primary — poor contrast on light |
| Pink | Youth, softness, subversion | DTC, social |
| Brown/tan | Craft, earthiness, heritage | Artisan goods, editorial |
| Teal | Clinical calm, modern | Health tech |

Use this table as a map of what's expected, then decide whether to meet or subvert the expectation. Deliberately going against association is a legitimate move when the brief supports it.

### 2.8 Known AI-design tells

These combinations currently signal machine-generated work. Avoid unless the brief specifically requests them:

- Cream background (~#F4F1EA) + high-contrast serif + terracotta accent (~#D97757)
- Near-black background + single acid-green or vermilion accent
- Purple-to-blue gradients on cards and buttons
- Glassmorphism applied without a reason for translucency
- Broadsheet layout with hairline rules, zero radius, and dense columns

Each of these is legitimate for *some* brief. None of them should be your answer when the brief leaves color free.

---

## 3. Typography

### 3.1 Type is the personality

The typeface is the loudest non-color decision on the page. A generic type pairing makes a design feel templated faster than any other single factor.

**Roles to fill:**
- **Display** — headlines, the hero. Characterful. Used sparingly and large. This is where personality lives.
- **Body** — sustained reading. Must be boring in the best sense: high legibility, generous x-height, unambiguous letterforms, real italics, multiple weights.
- **Utility** — captions, labels, data, code. Often a monospace or a condensed grotesque. Optional but useful.

Two families is the working default. Three is possible with discipline. One family across everything works if it has enough range in weight and width — pair a heavy display cut with a regular text cut of the same family.

**Pairing principles:**
- Contrast in *structure* (serif vs sans, geometric vs humanist) reads as intentional
- Similarity in structure with slight differences reads as a mistake
- Shared skeletal proportions (x-height, width) make an odd pairing cohere
- Superfamilies (a serif and sans designed together) are a safe cheat

**Classification quick reference:**
- *Old-style serif* — organic, angled stress, warm. Editorial, literary.
- *Transitional serif* — sharper, vertical stress. Institutional, authoritative.
- *Didone* — extreme thick/thin, hairline serifs. Fashion, luxury. Fragile at small sizes.
- *Slab serif* — heavy rectangular serifs. Sturdy, mechanical, confident.
- *Grotesque sans* — early sans, slightly irregular. Neutral with character.
- *Neo-grotesque* — Helvetica lineage. Clean, cold, ubiquitous.
- *Geometric sans* — circular forms. Modern, friendly, poor for long text.
- *Humanist sans* — calligraphic influence. Warmest and most readable sans.

### 3.2 Scale

Derive sizes from a ratio rather than choosing arbitrarily. Common ratios: 1.2 (minor third, tight), 1.25 (major third), 1.333 (perfect fourth), 1.5 (perfect fifth, dramatic).

Example at 16px base, 1.25 ratio:
```
12   caption, fine print
14   secondary, labels
16   body
20   lead paragraph, h4
25   h3
31   h2
39   h1
49   display
61   hero
```

Round to whole pixels. Six to eight sizes is enough for almost any interface — more than that and the scale stops reading as a system.

At display sizes, tighten letter-spacing (-1% to -3%) and line-height (1.0–1.15). At small sizes, loosen letter-spacing slightly and increase line-height. The mathematically scaled version of a headline is never quite right; adjust optically.

### 3.3 Measurable settings

- **Line length:** 45–75 characters for body; 60–66 is optimal. Use `max-width: 65ch`.
- **Line height:** 1.5–1.7 for body, 1.1–1.25 for headings, tighter as size increases. Longer lines need more line height.
- **Paragraph spacing:** 0.75–1.5× the line height. Use spacing *or* indentation, never both.
- **Letter-spacing:** default for body. Negative for large display. Positive (5–10%) for uppercase and small caps only.
- **Weight jumps:** skip a step for clear differentiation — 400 to 600, not 400 to 500. Adjacent weights read as an accident.
- **All caps:** short labels only. Always add letter-spacing. Never for more than a few words.

### 3.4 Hierarchy without size

Size is the blunt instrument. The refined tools:
- Weight — the most reliable secondary signal
- Color/value — a lighter gray demotes without changing size
- Case — small caps and uppercase for labels
- Spacing — isolation elevates
- Style — italic for asides and citations
- Family — switching to the utility face marks a different kind of content
- Rules and dividers — structural, not decorative

A hierarchy built from three of these is stronger and quieter than one built from size alone.

---

## 4. Components and surfaces

### 4.1 Elevation

Pick one elevation model and hold it across the product:
- **Shadow** — literal depth. Keep shadows soft, low-opacity (0.04–0.12), and offset downward only. Use 3–4 defined levels, never arbitrary values. Tint shadows with the background hue rather than pure black.
- **Border** — flat, precise, dense-friendly. Good for tools and data.
- **Fill** — surfaces differ by background lightness. Best for dark mode.

Mixing models within one interface is the most common visual inconsistency in amateur work.

### 4.2 Corner radius

Radius is a tone control. 0px reads technical and serious; 4–8px reads neutral and professional; 12–20px reads friendly and consumer; fully round reads playful or pill-like.

Rules:
- Pick 2–3 values maximum (e.g. 6 / 12 / full) and map them to component scale — small controls get the small radius.
- Nested elements: inner radius = outer radius − padding. Equal radii on nested boxes look wrong.
- Radius should be consistent with the typeface. A geometric sans with sharp corners on rounded boxes is a mismatch.

### 4.3 Interactive states

Every interactive element needs all six defined:
1. **Default**
2. **Hover** — subtle; a small lightness shift or elevation change
3. **Active/pressed** — should feel like depression: darker, smaller shadow, or 1px translate
4. **Focus** — a visible ring at 3:1 contrast, offset 2px from the element. Never remove it. This is an accessibility requirement, not a style choice.
5. **Disabled** — reduced contrast, `cursor: not-allowed`. Should look inert, not just faded.
6. **Loading** — for anything async. Skeletons over spinners when the shape is known.

### 4.4 Empty, error, and loading

These are design surfaces, not afterthoughts:
- **Empty state** — explain what goes here and give the action that fills it. An invitation, not an apology.
- **Error** — what happened, why, what to do next. In the interface's voice. Never vague, never apologetic, never blaming the user.
- **Loading** — skeleton screens matching final layout prevent the shift. Anything over ~1s needs progress indication.

---

## 5. Motion

Motion should explain something (origin, causation, relation) OR contribute subtle life to a static composition. The failure states are equal-and-opposite: **decorative motion without purpose reads as gratuitous**, but **a design with zero motion in 2026 reads as flat, unfinished, and dated**. Aim for the middle: subtle ambient motion + intentional interaction feedback.

**Duration:**
```
100–150ms  micro (hover, toggle, color shift)
200–300ms  standard (dropdown, tooltip, small reveal)
300–500ms  large (modal, page transition, drawer)
6–15s      ambient (background drift, gradient shift, orbital rotation) — loop
> 500ms    only for deliberate, orchestrated moments
```

**Easing:**
- `ease-out` for entering — fast start, gentle settle. The default for most UI.
- `ease-in` for exiting
- `ease-in-out` for elements moving between two on-screen positions
- Linear only for continuous motion (spinners, progress, ambient loops)
- Spring physics for anything that should feel physical or draggable
- Custom cubic-beziers for signature moments — e.g. `cubic-bezier(0.16, 1, 0.3, 1)` for a slow-in-fast-out reveal that feels considered

**Core principles:**
- Animate `transform` and `opacity` only. Everything else costs layout.
- Stagger sequential items by 30–50ms. More than ~6 staggered items feels slow.
- One orchestrated moment beats six scattered effects.
- Always honor `prefers-reduced-motion` — replace movement with an opacity fade, don't just disable.

### 5.1 Subtle ambient motion — the polish signal

These are what separate a polished 2026 design from a flat template. All must be subtle enough that removing them wouldn't confuse the composition — only diminish it.

**Always include at least three of these:**

- **Scroll-triggered fade-in** on each section: `opacity 0 → 1` + `translateY(16–24px) → 0` over 400–600ms with `ease-out`, staggered by 60ms across siblings, triggered via `IntersectionObserver`. Never animate more than 6 items in a single burst.
- **Hover lift** on cards / interactive tiles: `translateY(-2px)` + shadow deepen over 150ms `ease-out`. On release, spring back.
- **Focus emphasis**: 2px offset outline + subtle `scale(1.02)` on `:focus-visible`, 120ms transition. Accessibility floor and polish signal in one.
- **Ambient gradient drift** on hero backgrounds: slowly shift `background-position` or a CSS custom property over 12–15s, `ease-in-out`, infinite alternate. Amplitude never exceeds 4% of viewport.
- **Text reveal** on hero headline: word-by-word or line-by-line `opacity 0 → 1` + `translateY(8px) → 0`, staggered 40ms, 500ms `cubic-bezier(0.16, 1, 0.3, 1)`, fired once on page load.
- **Ghost cursor / underline** on primary links: `::after` pseudo with `scaleX(0) → scaleX(1)` from left origin on hover, 200ms `ease-out`.
- **Number ticker** on stats: count up from 0 to final on scroll-into-view over 800ms. `IntersectionObserver` + `requestAnimationFrame`.

**Never:**
- Elements that bounce, spin, or wiggle without cause
- Auto-playing videos with sound
- Parallax that fights natural scroll velocity
- Loading spinners that appear before a 500ms threshold — use skeletons
- More than one orchestrated moment per screen
- Motion on essential text (headings that jitter, body copy that fades continuously)

### 5.2 Three.js and 3D signature moments

A single 3D scene, when it fits the subject, transforms a design from "clean" to "memorable" — often the single biggest quality delta available. But it must serve the composition, not decorate it.

**When 3D belongs:**
- Subject has intrinsic depth: physics, astronomy, space, VR, 3D modelling, immersive audio, architecture, spatial computing
- Metaphorical subject: launch (rockets), growth (orbital rotation), depth (layers), atmosphere (particle fields), motion (fluid dynamics), scale (planetary bodies)
- Hero decoration where a static image would feel too literal
- As the signature element (per section 6) for any subject where "modern polish" is a brand goal — even a mundane CRM can earn a subtle particle field behind the hero if it's the only bold moment

**When it doesn't:**
- Forms, dense dashboards, data-heavy content pages — 3D competes with information
- Anything a screen reader must access — never for content or navigation
- More than one 3D scene per page. Always exactly one.
- Bento cell overlays — the tiling metaphor breaks
- Over key text without a `<div>` with opacity < 0.5 between them

**Canonical patterns (pick exactly one):**

1. **Floating gradient orb** — low-poly icosahedron or dodecahedron behind hero, palette-matched emissive/basic material, slow rotation (0.001 rad/frame) + subtle scale pulse (`0.95 → 1.05` over 8s). Compose with `backdrop-filter: blur(30px)` behind hero copy.
2. **Particle drift** — `BufferGeometry` with 800–1500 `Points`, `AdditiveBlending`, opacity 0.3–0.5, subtle wind-like drift on x-axis. Atmosphere, not focus.
3. **Wireframe object** — `TorusKnot` / `Dodecahedron` with wireframe material, line weight matched to typography stroke. Slow yaw rotation. Feels editorial + technical.
4. **Gradient plane** — `ShaderMaterial` on a plane, mixing two palette colors with slow simplex noise. Reads as an animated background wash. Palette-locked.
5. **Literal 3D illustration** — rockets, astronauts, orbital paths, planetary bodies. Use when the subject demands it or the reference site has one. Redraw matching silhouettes from the reference's inline SVG in three.js geometry.
6. **Scroll-driven camera path** — camera moves along a curve as user scrolls, revealing scene elements. Reserved for high-ambition marketing pages; consumes attention aggressively.

**Load pattern:**
```html
<script type="importmap">
{"imports": {"three": "https://cdn.jsdelivr.net/npm/three@0.170.0/build/three.module.js"}}
</script>
<script type="module">
  import * as THREE from "three";
  // scene, camera, renderer with alpha:true, antialias:true, powerPreference:"high-performance"
  // devicePixelRatio capped at 2
  // requestAnimationFrame loop that pauses when document.hidden
  // prefers-reduced-motion: freeze animation, don't remove the visual
</script>
```

**Performance and accessibility floor:**
- Cap `devicePixelRatio` at `Math.min(devicePixelRatio, 2)` — higher wastes battery for no perceived gain
- Pause `requestAnimationFrame` when `document.hidden`
- Honor `prefers-reduced-motion` — hold the still frame at initial state, don't strip the visual (the composition still needs it)
- The scene must survive removal — no critical content lives inside the WebGL canvas
- Initial state at t=0 must look intentional — never depend on interaction to render
- Canvas host is `position: absolute` + `pointer-events: none` so it can't block interaction

### 5.3 Motion as an antidote to "AI slop"

Static, quiet designs generated by AI look identical to each other. **Subtle motion is one of the strongest anti-slop signals** — it announces a designer decided how the page should feel. A design without any motion in 2026 self-identifies as generated. A design with too much motion self-identifies as amateur. The middle is: **three subtle ambient patterns + one signature moment (2D or 3D)**, all honoring reduced-motion.

---

## 6. The signature element

Every memorable design has one thing you'd describe if asked what it looked like. Not a style, a specific element: a type treatment, an interaction, a structural device, a data visualization, an illustration system, an unusual navigation, **a three.js scene (see 5.2) matched to the subject**.

**Rules:**
- Exactly one. Two signatures is zero signatures.
- It must come from the subject's own world — its materials, instruments, vocabulary, artifacts. A typography tool whose section dividers are baseline rules. A finance tool whose numbers use a tabular face with a ledger rule. A space or launch subject whose hero is a slow-rotating three.js orbital scene. This is where distinctiveness actually comes from.
- Everything around it stays disciplined and quiet. Boldness spent everywhere is boldness spent nowhere.
- It should survive removal of color. If it only works because of an accent hue, it's decoration.
- 3D signatures follow the same rule — the composition must still work if the WebGL scene fails to load.

---

## 7. Working process

**1. Brief interrogation.** Name subject, audience, single job. State assumptions where the brief is silent.

**2. Token plan, written before any code:**
- *Color:* 4–6 named hex values with roles
- *Type:* the families for display / body / utility, and why this pairing for this subject
- *Layout:* one-sentence concept plus an ASCII wireframe
- *Signature:* the one memorable element

**3. Critique the plan against the brief.** For each token, ask: is this specific to this brief, or is it what I'd produce for any brief in this category? Revise what fails. Name what changed and why.

**4. Build** to the revised plan exactly. Derive every value from the tokens.

**5. Critique the build:**
- Squint test — does hierarchy survive blur?
- Grayscale test — does it work without color?
- Mobile test — does it work at 375px?
- Content test — does it survive 2× and 0.5× the expected text length?
- Removal test — what can be deleted without loss? Delete it.

**6. Remove one thing.** There is always one accessory too many.

---

## 8. Quality floor

Non-negotiable regardless of aesthetic direction:

- Responsive to 320px without horizontal scroll
- Visible keyboard focus on every interactive element
- Contrast ratios verified for actual shipped pairs
- `prefers-reduced-motion` respected
- Touch targets ≥44×44px
- Semantic HTML; ARIA only where semantics are insufficient
- No layout shift on load — reserve space for images and async content
- All interactive states defined
- Empty, error, and loading states designed
- Real content tested, not lorem ipsum

---

## 9. Failure catalog

| Symptom | Cause | Fix |
|---|---|---|
| Feels cluttered | Insufficient whitespace, no hierarchy | Increase spacing between groups; demote elements |
| Feels flat | No contrast range in type or value | Widen the type scale; widen the neutral ramp |
| Feels amateurish | Arbitrary spacing values | Impose a spacing scale from a base unit |
| Feels muddy | Too many hues at similar saturation | Cut to one accent; desaturate the rest |
| Feels generic | Default type pairing and default palette | Rederive both from the subject |
| Feels chaotic | Too many alignment edges | Reduce to 2–3 vertical edges |
| Nothing draws the eye | Everything emphasized | Demote 80% of it |
| Cards look wrong | Uniform internal/external spacing | Internal padding < external gap ratio broken |
| Dark mode looks harsh | Inverted rather than rebuilt | Dark gray base, desaturated accents, 90% white text |
| Text hard to read | Line length or line height | Cap at 65ch, line-height 1.5–1.7 |
| Bento looks like a card grid | No dominant cell, uniform content | One 2×2 hero cell; vary content type per cell |
| Animation feels gratuitous | Motion without explanatory purpose | Cut everything that doesn't show causation or origin |

---

## 11. Anti-slop mastery — the difference between AI-generated and designed

The single biggest failure mode of AI-generated design is that it looks AI-generated. Every taxonomic tell in this section is paired with a concrete antidote. Reference this section whenever producing a hero, features section, or full page.

### 11.1 Structural slop signals

**The Big Six symmetry.** Almost every AI marketing page follows the same order: hero → 3-column features → testimonials → 3-tier pricing → CTA → footer. Break it by:
- Cutting testimonials or folding them into feature cells (bento-style)
- Skipping pricing entirely when the subject doesn't need it
- Making features a 1-big-4-small bento instead of 3 equal columns
- Slipping a case study, roadmap, or timeline BEFORE features
- Ending on a specific action (calendar embed, live console, sandbox) instead of another CTA band

**Perfect grid symmetry.** When everything is a 3×2 or 4×3 with all cells the same size, the page reads as "generated." Break it by:
- One dominant cell (2×2 hero cell in a bento)
- Asymmetric splits (60/40, 40/60, 30/70) instead of 50/50
- One element crossing the seam — image bleeding out of a column, headline breaking across cells
- Varying content type per cell — a number, a quote, an image, a chart — not the same icon+heading+text pattern in every cell

**Every heading centered.** Alternate alignment. Left-aligned headings after a centered hero read as considered. Centered eyebrows over left-aligned headings feel editorial. Center everything and the design feels lazy.

### 11.2 Visual slop signals + antidotes

| Slop signal | Antidote |
|---|---|
| Cream (#F4F1EA) + serif + terracotta (~#D97757) accent — the current cliché | Pick a palette from the subject's world (photographers → matte black + camera-bag brown; devs → paper + terminal green; chefs → tomato + basil + salt) |
| Purple-to-blue gradient hero | One considered palette color; add depth via lightness variation and shadow, not chroma shifts |
| Rounded corners at 12–16px on every card | 3 radius values mapped to component scale (small 4, med 8, large 16) — hold them consistently, don't apply the biggest one to everything |
| Soft drop shadow on every element | ONE elevation model (shadow, border, or fill). Max 3 shadow depths. Tinted with hue, never pure black |
| Every icon in accent-color pastels | Icons at ink color. Accent only for the one signature moment |
| Glassmorphism (backdrop-filter blur) with no subject justification | Use only if the subject involves transparency/depth as metaphor (windows, layers, atmosphere, water) |
| Line-art icons from Heroicons everywhere | Skip icons; use numbered eyebrows or one custom mark. Or match icon weight to typography stroke |
| Symmetric everything | One deliberate asymmetry per section (weight, alignment, color, motion) |

### 11.3 Content slop signals + antidotes

**Slop hero copy** (memorize these — they are DEAD):
- "Beautiful. Powerful. Simple."
- "The future of X"
- "Where teams do their best work"
- "Streamline your workflow with AI"
- "Unlock your potential"
- "The all-in-one X"
- "X, reimagined"
- "AI-powered X for the modern team"

**Impeccable hero copy** is a specific sentence about what this actually does, with a concrete noun the audience recognizes:
- "Time-boxed 20-minute writing sessions that ship one article per week"
- "Type an idea, get back a prototype your engineers can actually merge"
- "The scheduling tool photographers use because their clients text, not email"
- "Invoicing built for the 3-day payment cycle of catering companies"

Rules:
- Include one concrete noun from the audience's world ("invoice", "sessions", "case files", "voice memos", "wedding", "batch")
- Cut every abstract benefit word ("empowerment", "revolutionary", "seamless", "cutting-edge")
- If you can't say what it does in 12 words for a specific person, the hero copy is wrong

**Slop features:** three cards of "AI-Powered · Fast · Secure" with icon+heading+paragraph, three times.

**Impeccable features:**
- ONE dominant feature explained in genuine depth (real screenshot, annotated diagram, live code sample)
- 3–4 supporting facts as VARIED content types — one stat card, one testimonial line, one screenshot, one code block, one price/value pair
- Real product terminology, not generic verbs ("assign shift" not "manage tasks"; "reconcile invoice" not "streamline operations")

**Slop testimonials:**
- "This changed how we work" — Sarah, Founder, StartupCo
- Three interchangeable quotes that could be about literally any product

**Impeccable testimonials:**
- Specific outcome with real numbers: "We cut invoicing time from 2 hours to 12 minutes per month"
- Named person + realistic role + realistic company + realistic city: "Emma Chen, Studio Manager at Cornerstone Photo, Brooklyn"
- Attribution ties to a story: "Emma switched from Google Sheets after doubling her wedding volume"

**Slop CTAs:**
- "Get started" / "Start free trial" / "Sign up" / "Learn more"

**Impeccable CTAs** — write the actual first action:
- "Book 20 min with the founder"
- "Import your invoices"
- "Try it with your team's data"
- "Sync your Notion workspace"
- "Send us a test file"

Two-word verb-noun, not one-word CTAs.

**Slop stats:**
- "10x faster" / "100+ integrations" / "99.9% uptime" — round numbers everyone uses

**Impeccable stats:**
- "4 hours saved per week — median"
- "37% conversion lift on the checkout page"
- "500-page docs indexed in under 8 seconds"
- Odd numbers > round numbers. Specific > vague. Audit trail > superlatives.

**Slop trust:**
- "Trusted by teams at" + row of generic muted logos

**Impeccable trust:**
- One well-known logo with a link to a specific customer story
- OR a specific attribution: "In Q3 2025 we processed 3.2M invoices for 480 studios"
- OR skip the trust bar entirely — a great hero + product screenshot is more trustworthy than fake logos

### 11.4 Typography slop signals

| Slop | Antidote |
|---|---|
| Instrument Serif + Inter (2026 cliché) | Try Fraunces + Inter, Editorial New + General Sans, IBM Plex Serif + IBM Plex Sans, GT Sectra + Söhne, Karla + Fraunces |
| Body copy always Inter 400, line-height 1.5, nothing else | Occasional weight 500 for emphasis; vary line-height (headings 1.15, body 1.6, small 1.4); use italic for asides |
| Uppercase small-caps eyebrows over every section | Use once or twice per page — three or more is slop |
| Every heading centered | Left-align by default; center for one deliberate moment (hero) |
| Same weight everywhere | Mix 400 body + 500 emphasis + 600 for one microcopy signal (labels, callouts) |
| Symmetric font pairing (display serif + body sans — the safe default) | Try grotesque display + humanist sans; mono display + serif body; a single family with wide weight range; or slab + serif for editorial-technical |
| Text-align: justify on paragraphs | Ragged-right — justification without hyphenation creates rivers |

### 11.5 Motion slop signals

| Slop | Antidote |
|---|---|
| Every card lifts 4px on hover with the same 200ms transition | Vary the hover targets — one card gets depth, another a color shift, one has a subtle glow, one reveals a hidden detail |
| Everything fades in on scroll with 60ms stagger | Reserve scroll-fade for the hero + one section; skip it elsewhere so the intentional ones feel intentional |
| No motion at all | See §5.3 — zero motion also reads as generated |
| Gradient shift on hero background looping every 4 seconds | Longer, subtler cycles (12–15s) with lower amplitude; or replace with a three.js signature |
| Bounce/spring on everything | Reserve spring physics for interactions where physicality matters (drag, drop, expand) |

### 11.6 The impeccable-design final checks

Before shipping any design, apply these five tests. All five must pass.

1. **The specificity test.** For every visible decision, ask "would I have made the same choice for a completely different subject?" If yes, it's a default, not a decision. Replace with something derived from THIS subject.

2. **The removal test.** Delete every third element. Which does the composition survive without? Delete those permanently. What remains is the design.

3. **The mimicry test.** Would a human designer who has really thought about this problem have made these exact choices? If it looks like anyone-anywhere could have written it, it's slop.

4. **The one-line description test.** Can you describe what makes this design distinctive in one specific sentence? "It has a serif headline and a warm palette" is slop. "The pricing table is drawn as a receipt with hand-torn edges because the product handles freelance invoices" is not.

5. **The grayscale test.** Strip all color. Is the design still legible, still hierarchical, still distinctive? If it flattens to nothing, color was doing all the work — a slop tell.

### 11.7 The one-signature rule (restated)

Every impeccable design has EXACTLY ONE moment that couldn't exist elsewhere. Not two. Not everywhere. One.

- A finance app whose numbers use a tabular font with a ledger rule at every $10,000 mark
- A typography tool whose section dividers are actual baseline rules
- A meditation app whose CTA button is a breathing circle that expands and contracts on hover
- A photography scheduling tool whose calendar days are Polaroid frames
- A code editor whose testimonials are commit messages
- A cooking app whose feature icons are actual knife strokes

Everything else stays quiet and disciplined. Boldness spent everywhere is boldness spent nowhere.

---

## 12. Grid systems in depth

### 12.1 Column count decision

| Columns | When to use | Gutter typical |
|---|---|---|
| 12 | Marketing pages, general web — most flexible | 24–32px |
| 8 | Simpler layouts, editorial features | 24–32px |
| 6 | Editorial, magazine, product pages | 32–48px |
| 5 | Odd count creates asymmetric tension — editorial only | 24–32px |
| 4 | Bento grids, dashboards, dense tools | 16–24px |

Rule: fewer columns = more character. 12-column reads as flexible/generic; 4–6 columns reads as decisive.

### 12.2 Gutter and margin math per breakpoint

```
Viewport width   Margin      Gutter    Max content width
< 640px          16–20px     12px      full - margins
640–1024px       24–32px     16px      full - margins
1024–1280px      40–48px     20–24px   1200px cap
1280–1600px      64–80px     24px      1200–1280px cap
> 1600px         grows       24px      1280–1440px cap
```

Beyond 1440px content width, additional viewport becomes margin, not content. Long lines destroy reading.

### 12.3 Baseline grid setup (for editorial)

- Choose a base line-height (typically 24px for 16px body)
- ALL vertical margins/padding = multiples of the base (24, 48, 72, 96)
- Headings sized so their line boxes = integer multiples of base
- Optical adjustment: -1–2px on headings to counteract type metrics

Result: every text baseline aligns across columns. Slow to author, striking to read.

### 12.4 Modular grid math

Work backward from container width:
```
Cell size = (container_width - (columns - 1) × gutter) / columns
Example: 4 columns, 24px gutter, 1200px container
        = (1200 - 72) / 4 = 282px per cell
        = 2×2 hero cell: 282×2 + 24 = 588px wide
```

If cells don't come out to whole pixels, adjust gutter until they do.

### 12.5 Broken grid — when and how

Deliberate off-column placement creates dynamism. Rules:
- Exactly ONE anchor element aligned to the grid, so the eye has a reference point
- Break by 1–2 columns' worth, not tiny half-column shifts (which read as mistakes)
- Break by full cell heights, not arbitrary vertical offsets
- Editorial and portfolio contexts only — never in app UI or dashboards

### 12.6 Compound grids

Different sections can use different grids on the same page:
- Hero: 5-column broken grid with headline breaking column 2
- Features: 4-column bento with 2×2 dominant cell
- Testimonials: 3-column card grid
- Pricing: 12-column with three equal spans
- Footer: 6-column sitemap

Compound grids work when transitions between sections have adequate breathing room (48–96px vertical) and each section's grid is decisively chosen.

---

## 13. Tone archetypes — palette + type + spacing + motion recipes

Each tone is a coherent package. Mix and match individual axes deliberately; don't blend two archetypes across the same design.

### 13.1 Professional SaaS

- **Palette**: Neutrals + one deep-blue accent (#1e40af family), optional secondary teal/green
- **Type**: Inter or Söhne body; occasional serif for one editorial moment
- **Scale**: 1.25 ratio, 6 sizes, weight jumps 400→600
- **Spacing**: 4px base, generous but restrained (32–48px between sections)
- **Radius**: 6–8px cards, 4px inputs, 4px buttons
- **Motion**: 150ms hover, 250ms transitions, subtle scroll fades
- **Density**: Standard (40–44px rows)
- **Signature**: A clean data visualization, or a hyper-specific product screenshot with annotation

### 13.2 Editorial

- **Palette**: Warm neutrals + one saturated accent (brick, ochre, forest, ink-blue)
- **Type**: Serif display (Instrument, Fraunces, Editorial New, GT Sectra) + humanist sans (Inter, Söhne, Karla)
- **Scale**: 1.333 or 1.5 ratio, wide range, italic emphasis
- **Spacing**: 8px base, generous margins (64–96px section separation)
- **Radius**: 0–4px max; hairline (1px) borders and rules do the visual work
- **Motion**: Subtle fade-ins, hover lift, text reveal on load
- **Density**: Comfortable (48–56px)
- **Signature**: Pull quotes, hanging punctuation, drop caps, section-numbered eyebrows

### 13.3 Playful consumer

- **Palette**: Vibrant saturated primary + secondary accent + soft muted supporting colors
- **Type**: Rounded geometric sans (Nunito, Poppins, DM Sans) or one custom display face
- **Scale**: 1.25–1.333 ratio
- **Spacing**: 8px base, tight-to-medium (24–32px sections)
- **Radius**: 12–24px, occasionally fully round (pill buttons, avatar circles)
- **Motion**: Spring physics on interactive elements, bounce on card entry, animated illustrations
- **Density**: Comfortable
- **Signature**: Illustration system, characters, hand-drawn accents

### 13.4 Technical / developer

- **Palette**: Dark charcoal (#0e0e10) + off-white text + one saturated accent (mint, cyan, or brand)
- **Type**: Mono for display AND code (JetBrains Mono, Berkeley Mono, IBM Plex Mono) + grotesque sans for body
- **Scale**: 1.2 ratio, compact
- **Spacing**: 4px base, tight (24–32px between elements)
- **Radius**: 2–4px max, often 0px
- **Motion**: Minimal — instant feedback, no fades
- **Density**: Compact (28–32px)
- **Signature**: Terminal-inspired hero (blinking cursor, command palette), inline code samples

### 13.5 Luxury

- **Palette**: Deep neutrals (near-black, ivory) + one metallic-inspired accent (bronze, brass, oxblood, forest)
- **Type**: Didone or transitional serif (Playfair, GT Sectra, Bodoni) + refined sans (Neue Haas, GT America)
- **Scale**: 1.5+ ratio for hero, dramatic contrast
- **Spacing**: Extremely generous (96–128px between sections, 48–64px within)
- **Radius**: 0px — sharp precision
- **Motion**: Slow, considered (500–800ms), everything on ease-in-out
- **Density**: Comfortable, sometimes spacious
- **Signature**: Full-bleed cinema imagery, hairline gold rules, wide letter-spaced small caps

### 13.6 Warm hospitality / consumer

- **Palette**: Warm earth tones (tan, terracotta, forest, cream) + one saturated accent
- **Type**: Humanist sans (Karla, Söhne, General Sans) + occasional serif italic (Fraunces) for warmth
- **Scale**: 1.25 ratio
- **Spacing**: 8px base, medium (32–48px)
- **Radius**: 8–16px, soft
- **Motion**: Gentle 200–300ms transitions, subtle animations
- **Density**: Comfortable
- **Signature**: Photography-forward, hand-lettering, warm textured backgrounds

### 13.7 Clinical / healthcare

- **Palette**: Cool neutrals + teal/mint accent + optional warm secondary
- **Type**: Humanist sans (Inter, Karla) with clear letterforms; avoid geometric or condensed
- **Scale**: 1.2–1.25 ratio, restrained
- **Spacing**: 8px base, generous (48–64px sections) for calmness
- **Radius**: 6–8px, gentle
- **Motion**: Slow and subtle (300–400ms, ease-in-out); nothing sudden
- **Density**: Comfortable
- **Signature**: Precise data visualization, thermal-line dividers, translucent depth layers

### 13.8 Brutalist / anti-design

- **Palette**: Black + white + ONE saturated primary (electric red, cyber green, hot pink)
- **Type**: Condensed grotesque (Space Grotesk, Neue Haas Grotesk) or mono at display sizes
- **Scale**: Extreme range (8px caption to 96px display)
- **Spacing**: Irregular, deliberate — some elements crammed, others floating
- **Radius**: 0px — everything hard-edged
- **Motion**: None or aggressive (instant snap, no ease)
- **Density**: High
- **Signature**: Raw HTML aesthetic, unstyled elements as art, brutalist typography

---

## 14. Section anatomies — what makes each great

### 14.1 The impeccable hero

Structure:
- ONE headline, 8–14 words, specific and concrete (see §11.3)
- ONE subhead, 12–24 words, elaborating the specific benefit
- ONE primary CTA (verb-noun); optionally one secondary in ghost style
- ONE visual (product screenshot, hand-drawn diagram, three.js scene, hero image)
- Optional: attribution/trust chip with specific numbers (not "Trusted by 1000+")

Rules:
- Don't stack multiple CTAs (choice paralysis)
- Headline MUST contain a concrete noun from the subject's world
- If the visual is a screenshot, it must be READABLE at page load size — not a shrunken thumbnail
- Above-the-fold is dead — make the hero worth scrolling PAST, not "fit everything above"

### 14.2 The impeccable features section

**Bad**: three icon+heading+paragraph cards in a symmetric grid.

**Good** — bento with varied content types:
- ONE dominant cell (2×2) with the biggest most important claim, a screenshot or animated diagram
- 1–2 medium cells with supporting facts or stats
- 1–2 small cells with specific micro-features
- Every cell has a DIFFERENT content type — image, stat, quote, screenshot, icon+text, chart

**Good alternative** — alternating rows:
- Text left, image right; then image left, text right
- 4–6 rows max
- Each row explains ONE thing in real depth (paragraph copy, not bullets)

Skip icons unless they're custom-drawn for this product.

### 14.3 The impeccable testimonials

Structure:
- 3–5 specific quotes
- Named person + realistic role + realistic company + realistic city
- Optional: real photo (via pravatar) OR just initials in a circle
- Specific outcome numbers when possible
- Attribution ties to a real story

Layouts:
- Marquee scroll (continuous, hover to pause)
- Bento with one large quote + 3–4 smaller
- Card grid (only if quotes are genuinely peer-level)
- ONE hero quote with big attribution and a supporting screenshot

### 14.4 The impeccable pricing

Structure:
- 3 tiers default (Starter / Pro / Enterprise), 2 is fine, 4+ is confusing
- Middle tier "Recommended" with visual emphasis (border, badge, background)
- Each tier: name, price, one-line description, 5–7 SPECIFIC features, CTA
- Feature list: "10 GB storage" not "generous storage"

Alternative structures:
- Single plan with annual toggle
- Slider (usage-based pricing)
- Comparison matrix (only when features vary meaningfully)

Rules:
- Prices visible without hover
- "Contact sales" only for enterprise — never for all tiers
- Cancel policy clearly shown below the tiers

### 14.5 The impeccable CTA section

Structure:
- ONE specific action, not "get started"
- Optional supporting sentence
- Optional trust/urgency signal (specific: "Built by 3 photographers who ship weekly", not "Trusted by thousands")

Layouts:
- Split band (image left, action right)
- Centered boxed
- Full-bleed with contrasting background
- Inline with the final content section (not a separate CTA band)

Skip:
- "Ready to X?" cliché
- "Get started free" if there's no free tier
- Newsletter capture as the primary CTA on a product page

### 14.6 The impeccable footer

Structure:
- Brand mark (real logo treatment, not just wordmark of company name)
- Sitemap columns (Product / Company / Resources / Legal)
- Contact / address / social
- Copyright + specific attribution
- Optional: newsletter capture, language switcher

Rules:
- Legal in muted tone but readable — 12–14px, muted gray, not invisible
- Real social links, not empty icons
- Cite specific attribution: "Made in Brooklyn by 4 humans" beats "© 2026 Company Inc."
- Consider a giant wordmark as the signature treatment

---

## 15. Dashboards and product UIs — beyond the landing page

A dashboard is NOT a landing page. It has no hero, no CTA, no pricing tiers. It has a **shell** (sidebar/topbar), a **workspace** (the main area), and **widgets** (things that display or act on data). Get the shell right and everything else falls into place.

### 15.1 Shell archetypes

Pick ONE — do not combine two shell types in a single design.

**Sidebar shell** (Linear, Notion, Vercel dashboard, Stripe, Retool)
- Fixed 240–280px left rail, dark or tinted-neutral.
- Rail sections: workspace switcher (top), primary nav (middle), user/settings (bottom).
- Primary nav items: 6–10 max, grouped with dividers or headers, each with a 16–20px icon + label.
- Active state: subtle left border accent + slight background tint, never a full pill.
- Collapsible on desktop (rail → icon-only), hidden on mobile behind a menu.

**Topbar shell** (Attio, Airtable, Figma)
- 56–64px top nav, workspace switcher on the left, search center, user/notifications right.
- Secondary nav below (tabs) OR left rail below topbar (mixed shell).
- Works when horizontal breadth matters (spreadsheet-like tools, canvases).

**Mixed shell** (Linear Insights, Datadog, Notion databases)
- Topbar (48–56px) with workspace and account.
- Left rail (200–240px) for section navigation.
- Optional right rail (280–360px) for context / filters / details.
- Highest information density. Best for analytics + data tools.

**Rules for all shells:**
- Shell is present on every page of the app — same rail, same topbar, only workspace content changes.
- Nav labels are concrete nouns from the product's domain, not "Dashboard / Overview / Analytics" everywhere.
- Icon set is consistent (all outline, or all filled — never mixed) at consistent stroke weight.

### 15.2 Workspace patterns

The main content area follows a small set of patterns. Match to the data type.

**Grid of stat cards** — 3 to 6 KPI tiles across the top, showing single scalar values with delta and mini-trend. Below: a chart or table. Best for executive views.

**Chart-first** — one large chart or map dominates, controls and filters above/beside, related tables below. Best for analytics and observability.

**Table-first** — a data table fills the workspace, with filter bar above, row detail in a right rail or drawer. Best for CRUD tools (users, orders, tickets).

**Split canvas** — left column: list / navigation of items. Right column: detail of the selected item. Best for inbox-like tools (issues, threads, records).

**Editor canvas** — full-bleed canvas or textarea with floating toolbars, right rail for properties. Best for editors (Notion, Figma, Linear issue view).

**Card feed** — vertical list of cards with rich content. Best for social feeds, activity streams, notification centers.

### 15.3 Common widgets

**Stat card**
- 1 metric name (12px, muted, uppercase or normal case), 1 large number (32–40px, tabular numerals), 1 delta indicator with arrow + percentage + color (green up / red down / grey neutral), 1 optional sparkline/mini-chart below.
- 3–6 cards in a row on desktop, 1–2 per row on mobile.
- Padding: 20–24px inside; gap 16–20px between.
- No border chrome — use tinted-neutral background against paper.

**Data table**
- Sticky header row, subtle bottom-border rows (never full grid lines).
- Cell padding 12–16px vertical, 16–20px horizontal.
- Right-aligned numeric columns, tabular numerals.
- First column often bolder or wider (the "name" column).
- Row hover: slight tint of accent. Row click → open detail.
- Column headers sortable; show sort indicator.
- Empty state: illustration + one line + primary action, NOT a blank grid.
- Pagination or infinite scroll — pick one.
- Filter bar above: search + 2–4 filter chips + view switcher.

**Chart widget**
- Chart type matches data: line for time-series, bar for categories, donut only for 2–4 slices (never more), area for cumulative.
- Y-axis often unlabeled if number is self-evident. X-axis: dates in short form (Mon 12, Jan 3).
- No gridlines behind bars/lines unless data density demands it — one horizontal reference line at the baseline is enough.
- Legend inline with chart title, not as a separate block.
- Interactive tooltip on hover: date + value + delta from previous.
- Empty state: same rule as tables.

**Activity feed / timeline**
- Left-aligned timeline dot + line, right-aligned event copy.
- Group by day: "Today", "Yesterday", specific date.
- Each event: actor + verb + object + relative time ("Ankush closed issue #42 · 2h ago").
- Avatar or icon for the actor.

**Filter bar**
- Search input (with keyboard shortcut hint on the right — `⌘K`).
- 2–4 filter chips (each opens a popover): Status, Owner, Date range, Tag.
- View switcher on the right (list / grid / kanban).
- Sort dropdown after view switcher.
- Never crowd — if you need 6+ filters, use a slide-out filter panel triggered by an "All filters" button.

**Detail drawer / right rail**
- 360–480px wide, slides in from the right when a row/item is clicked.
- Header: item name, close button (top-right), primary actions (top-right or bottom-right).
- Body: property list (label:value pairs, label left aligned, value right aligned) then rich content sections.
- Footer: secondary actions or metadata (created / updated timestamps).

**Command palette (⌘K)**
- 480px centered modal overlay with 30% dim behind.
- Input at top, results below grouped by type (Actions / Navigation / Documents).
- Each result: icon + label + shortcut hint on right + secondary text.
- Keyboard-first — arrow keys navigate, enter runs, esc closes.

**Empty states**
- Every list, table, chart, panel needs one.
- Structure: illustration or icon (small, not a giant graphic) + one-line explanation + primary action button.
- Example: "No issues assigned to you yet. [Create issue]"
- Never blank space. Never "No data."

**Notification / toast**
- Bottom-right or top-right corner, 320–400px wide.
- Icon (info / success / warning / error, color-matched), title, optional detail line, close button, optional action link.
- Auto-dismiss after 4–6s for info/success, sticky for warning/error.

### 15.4 Density and hierarchy

Dashboards live at a different density than marketing pages. Get the math right:

- Base spacing unit: 4px (dashboards are denser than the 8px used for landing pages).
- Card padding: 16–24px (not 32–48px).
- Section spacing: 24–32px between panels (not 96–120px).
- Body text: 14px (not 16–18px).
- Line-height: 1.4–1.5 (not 1.6).
- Font weight jumps are smaller: 400 body → 500 emphasis → 600 headings (not 400 → 700).

Hierarchy without size:
- Muted text (60–70% of ink color) for labels.
- Tabular numerals for all numbers.
- Icons scale with text (never bigger than the accompanying label's cap-height + 20%).

### 15.5 The dashboard color system

- Paper (workspace background): near-white, warm off-white, or near-black.
- Rail (sidebar): tinted-neutral, slightly darker/lighter than paper. Never pure grey.
- Card surface: paper, or one step of elevation from paper (2–3% shift, not a shadow).
- Ink: main text, 90–95% of pure black or pure white.
- Muted ink: labels, timestamps, secondary text. 55–65% of ink.
- Accent: brand color, used ONLY for interactive elements and the ONE key data callout.
- Semantic: green (positive delta), red (negative delta), amber (warning). Distinct from accent.
- Chart colors: a set of 5–7 hues designed to sit on card surface — NOT the accent color.

### 15.6 When the freeform LLM path is asked for a dashboard

1. Choose a shell archetype (usually sidebar or mixed for admin tools).
2. Choose the primary workspace pattern (stat grid + chart + table is the most common).
3. Realistic domain nav labels ("Athletes / Workouts / Programs / Payments" for fitness, not "Users / Analytics / Reports").
4. Real data in every widget — never Lorem Ipsum, never placeholder numbers like "10K" or "99.9%". Use specific, believable, odd numbers: "1,847 active this week", "$28,412.53 pending payout".
5. Include at least one empty state somewhere so the design shows how it degrades.
6. Every nav item is a real `<a>` — do NOT use `href="#"`. Use `href="./settings.html"`, `href="./users.html"` — real filenames so the multi-page navigation flow can wire them up later.

### 15.7 Multi-page apps (roadmap concept — currently designs are single-page)

Real product UIs span multiple pages: Home → Settings → Users → Reports. Design intent:

- Design page 1 (Home) with a working nav pointing to real filenames.
- User clicks an unwired nav link on canvas → prompt: "Design the {settings} page?"
- If yes → new sub-design in the same project, inheriting the shell (sidebar, topbar, theme).
- Tab bar above canvas lets user switch between pages.
- Export packages all pages as a folder of linked HTML files.

Until multi-page ships, dashboards SHOULD use real relative-path filenames in nav (`./settings.html`, etc.) so that when multi-page ships, existing designs upgrade cleanly.

---

## 10. Compressed checklist

**Before designing:** subject, audience, job named.

**Color:** neutral ramp tinted with the primary hue; one accent; semantic set distinct from primary; 60/30/10 proportion; contrast verified; dark mode rebuilt not inverted.

**Type:** display and body chosen for this subject; ratio-derived scale of 6–8 sizes; 45–75ch measure; line-height 1.5–1.7 body; weight jumps skip a step.

**Layout:** reading order written first; spacing from a base unit; internal gaps < external gaps; 2–3 alignment edges; archetype matched to content; mobile reordered by importance.

**Components:** one elevation model; 2–3 radius values; six states defined; empty/error/loading designed.

**Motion:** at least three subtle ambient patterns (scroll fade-in, hover lift, focus emphasis, gradient drift, text reveal, or number ticker); transform + opacity only; reduced-motion honored; NEVER zero motion — that reads as unfinished.

**3D:** optional. When subject warrants depth (space, physics, growth, immersion) or reference site has 3D — include ONE three.js scene (floating orb / particle drift / wireframe object / gradient plane / literal illustration). Never more than one. Never over content or navigation. Loads via importmap, honors reduced-motion, composition survives if it fails.

**Signature:** exactly one, drawn from the subject, survives grayscale.

**Anti-slop (from §11):** no Big Six symmetric order; no purple-to-blue gradients or cream+serif+terracotta cliché; no "Beautiful. Powerful. Simple."-style hero; no interchangeable testimonials; no "Get started" CTA — write the actual first action; no round-numbered stats — cite specific odd numbers; one deliberate asymmetry per section; kill icon-heading-paragraph triples in favor of varied content types per bento cell.

**Grid (from §12):** column count picked decisively (12 flexible, 4–6 characterful); gutter/margin math per breakpoint respected; if bento, one dominant 2×2 cell + varied content types; if editorial, baseline grid; if broken grid, one anchor element aligned to grid as reference.

**Tone package (from §13):** pick ONE of professional / editorial / playful / technical / luxury / warm / clinical / brutalist. Do not blend two across the same design — palette, type, radius, motion, and density all move together as a coherent recipe.

**Section anatomies (from §14):** hero has ONE concrete-noun headline + ONE CTA + ONE visual; features vary content type per cell (not icon+heading+text ×6); testimonials cite specific outcomes; pricing has 3 tiers with real feature lists; CTA is a specific first action; footer cites specific attribution.

**Final 5 tests:** specificity, removal, mimicry, one-line description, grayscale. All five must pass.

**Last step:** remove one thing.
