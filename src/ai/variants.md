# Variant library

Hand-authored HTML fragments, one per section variant. Each entry has metadata, an optional `---STYLE---` block (CSS with palette custom properties), and an `---HTML---` block. Placeholders are `{{NAME}}` — the assembler fills them via a small LLM content-fill call.

All variants use palette custom properties (`var(--paper)`, `var(--ink)`, `var(--accent)`, etc.) so a palette swap re-tones every variant without editing HTML.

---

## nav-01: brand-heavy-serif
[category: navbar]
[tags: editorial, marketing]
[placeholders: BRAND_NAME, LINK_1, LINK_2, LINK_3, LINK_4, CTA_LABEL]
---STYLE---
.nav-01 { display: flex; align-items: center; justify-content: space-between; padding: 24px 48px; border-bottom: 1px solid var(--line); background: var(--paper); flex-wrap: wrap; gap: 16px; }
.nav-01 .mark { font-family: var(--font-display); font-size: 22px; font-weight: 400; letter-spacing: -0.01em; color: var(--ink); }
.nav-01 .links { display: flex; gap: 32px; list-style: none; margin: 0; padding: 0; }
.nav-01 .links a { color: var(--ink-2); text-decoration: none; font-size: 14px; transition: color 150ms; }
.nav-01 .links a:hover { color: var(--accent); }
.nav-01 .cta { background: var(--ink); color: var(--paper); padding: 8px 16px; border-radius: 4px; text-decoration: none; font-size: 13px; font-weight: 500; transition: background 150ms; }
.nav-01 .cta:hover { background: var(--accent); }
.nav-01 .burger { display: none; background: transparent; border: 1px solid var(--line); border-radius: 4px; padding: 8px 12px; cursor: pointer; font-size: 18px; color: var(--ink); font-family: inherit; }
@media (max-width: 768px) {
  .nav-01 { padding: 16px 20px; }
  .nav-01 .mark { font-size: 20px; flex: 1; }
  .nav-01 .burger { display: inline-block; }
  .nav-01 .links { display: none; flex-direction: column; gap: 4px; width: 100%; padding: 12px 0 4px; border-top: 1px solid var(--line); order: 10; }
  .nav-01 .links a { padding: 12px 0; font-size: 16px; }
  .nav-01 .cta { display: none; }
  .nav-01.open .links { display: flex; }
  .nav-01.open .cta { display: inline-block; margin-top: 8px; align-self: flex-start; padding: 12px 20px; font-size: 15px; }
}
---HTML---
<nav data-section="nav" data-variant="nav-01" class="nav-01">
  <span class="mark">{{BRAND_NAME}}</span>
  <button class="burger" aria-label="Toggle menu" onclick="this.parentElement.classList.toggle('open')">☰</button>
  <ul class="links">
    <li><a href="#features">{{LINK_1}}</a></li>
    <li><a href="#pricing">{{LINK_2}}</a></li>
    <li><a href="#about">{{LINK_3}}</a></li>
    <li><a href="#docs">{{LINK_4}}</a></li>
  </ul>
  <a class="cta" href="#cta">{{CTA_LABEL}}</a>
</nav>

---

## nav-02: sticky-transparent
[category: navbar]
[tags: modern, saas, sticky]
[placeholders: BRAND_NAME, LINK_1, LINK_2, LINK_3, CTA_LABEL]
---STYLE---
.nav-02 { position: sticky; top: 0; z-index: 100; display: flex; align-items: center; justify-content: space-between; padding: 16px 40px; background: rgba(255,255,255,0.85); backdrop-filter: blur(12px); border-bottom: 1px solid var(--line); flex-wrap: wrap; gap: 12px; }
.nav-02 .brand { display: flex; align-items: center; gap: 8px; font-family: var(--font-body); font-weight: 600; font-size: 16px; color: var(--ink); text-decoration: none; }
.nav-02 .brand::before { content: ""; display: block; width: 24px; height: 24px; background: var(--accent); border-radius: 6px; }
.nav-02 .links { display: flex; gap: 28px; list-style: none; margin: 0; padding: 0; }
.nav-02 .links a { color: var(--muted); text-decoration: none; font-size: 14px; font-weight: 500; }
.nav-02 .links a:hover { color: var(--ink); }
.nav-02 .cta { background: var(--accent); color: var(--paper); padding: 8px 14px; border-radius: 6px; text-decoration: none; font-size: 13px; font-weight: 500; }
.nav-02 .burger { display: none; background: transparent; border: 1px solid var(--line); border-radius: 6px; padding: 8px 12px; cursor: pointer; font-size: 18px; color: var(--ink); font-family: inherit; }
@media (max-width: 768px) {
  .nav-02 { padding: 12px 20px; }
  .nav-02 .brand { flex: 1; }
  .nav-02 .burger { display: inline-block; }
  .nav-02 .links { display: none; flex-direction: column; gap: 4px; width: 100%; padding: 8px 0 4px; border-top: 1px solid var(--line); order: 10; }
  .nav-02 .links a { padding: 12px 0; font-size: 16px; }
  .nav-02 .cta { display: none; }
  .nav-02.open .links { display: flex; }
  .nav-02.open .cta { display: inline-block; margin-top: 4px; align-self: flex-start; padding: 12px 20px; font-size: 15px; }
}
---HTML---
<nav data-section="nav" data-variant="nav-02" class="nav-02">
  <a class="brand" href="#top">{{BRAND_NAME}}</a>
  <button class="burger" aria-label="Toggle menu" onclick="this.parentElement.classList.toggle('open')">☰</button>
  <ul class="links">
    <li><a href="#features">{{LINK_1}}</a></li>
    <li><a href="#pricing">{{LINK_2}}</a></li>
    <li><a href="#docs">{{LINK_3}}</a></li>
  </ul>
  <a class="cta" href="#cta">{{CTA_LABEL}}</a>
</nav>

---

## nav-03: centered-editorial
[category: navbar]
[tags: editorial, luxury, minimal]
[placeholders: BRAND_NAME, LINK_1, LINK_2, LINK_3, LINK_4, CTA_LABEL]
---STYLE---
.nav-03 { display: grid; grid-template-columns: 1fr auto 1fr; align-items: center; padding: 20px 48px; border-bottom: 1px solid var(--line); }
.nav-03 .left { display: flex; gap: 24px; }
.nav-03 .left a { color: var(--muted); text-decoration: none; font-size: 12px; letter-spacing: 0.14em; text-transform: uppercase; }
.nav-03 .mark { font-family: var(--font-display); font-size: 28px; text-align: center; color: var(--ink); }
.nav-03 .right { display: flex; gap: 20px; justify-content: flex-end; align-items: center; }
.nav-03 .right a { color: var(--muted); text-decoration: none; font-size: 12px; letter-spacing: 0.14em; text-transform: uppercase; }
.nav-03 .cta { color: var(--accent) !important; border-bottom: 1px solid var(--accent); padding-bottom: 2px; }
.nav-03 .burger { display: none; background: transparent; border: 1px solid var(--line); border-radius: 4px; padding: 8px 12px; cursor: pointer; font-size: 18px; color: var(--ink); font-family: inherit; }
@media (max-width: 768px) {
  .nav-03 { display: flex; flex-wrap: wrap; justify-content: space-between; padding: 16px 20px; gap: 12px; }
  .nav-03 .mark { font-size: 24px; text-align: left; order: 1; flex: 1; }
  .nav-03 .burger { display: inline-block; order: 2; }
  .nav-03 .left, .nav-03 .right { display: none; flex-direction: column; align-items: flex-start; gap: 4px; width: 100%; order: 10; padding-top: 8px; border-top: 1px solid var(--line); }
  .nav-03 .left a, .nav-03 .right a { padding: 12px 0; font-size: 14px; letter-spacing: 0.12em; }
  .nav-03.open .left, .nav-03.open .right { display: flex; }
  .nav-03.open .right { border-top: none; padding-top: 0; }
}
---HTML---
<nav data-section="nav" data-variant="nav-03" class="nav-03">
  <div class="left">
    <a href="#features">{{LINK_1}}</a>
    <a href="#pricing">{{LINK_2}}</a>
  </div>
  <span class="mark">{{BRAND_NAME}}</span>
  <button class="burger" aria-label="Toggle menu" onclick="this.parentElement.classList.toggle('open')">☰</button>
  <div class="right">
    <a href="#about">{{LINK_3}}</a>
    <a href="#docs">{{LINK_4}}</a>
    <a class="cta" href="#cta">{{CTA_LABEL}}</a>
  </div>
</nav>

---

## hero-01: centered-editorial
[category: hero]
[tags: editorial, marketing]
[placeholders: EYEBROW, HEADLINE_PART_A, HEADLINE_ITALIC, HEADLINE_PART_B, SUBHEAD, CTA_PRIMARY, CTA_SECONDARY, HERO_IMAGE_URL, HERO_IMAGE_ALT]
---STYLE---
.hero-01 { padding: 96px 48px 64px; text-align: center; max-width: 1200px; margin: 0 auto; }
.hero-01 .eyebrow { font-size: 12px; font-weight: 600; letter-spacing: 0.2em; text-transform: uppercase; color: var(--accent); margin-bottom: 16px; }
.hero-01 h1 { font-family: var(--font-display); font-size: clamp(40px, 6vw, 72px); font-weight: 400; line-height: 1.05; letter-spacing: -0.015em; color: var(--ink); margin: 0 0 24px; max-width: 800px; margin-left: auto; margin-right: auto; }
.hero-01 h1 em { font-style: italic; color: var(--accent); }
.hero-01 .sub { font-size: 18px; line-height: 1.5; color: var(--muted); max-width: 620px; margin: 0 auto 32px; }
.hero-01 .ctas { display: flex; gap: 12px; justify-content: center; margin-bottom: 64px; }
.hero-01 .cta-primary { background: var(--ink); color: var(--paper); padding: 12px 24px; border-radius: 4px; text-decoration: none; font-size: 14px; font-weight: 500; transition: background 150ms; }
.hero-01 .cta-primary:hover { background: var(--accent); }
.hero-01 .cta-secondary { color: var(--ink-2); padding: 12px 12px; text-decoration: none; font-size: 14px; font-weight: 500; border-bottom: 1px solid transparent; }
.hero-01 .cta-secondary:hover { border-bottom-color: var(--ink-2); }
.hero-01 img { width: 100%; max-width: 1100px; border-radius: 8px; border: 1px solid var(--line); }
@media (max-width: 640px) {
  .hero-01 { padding: 48px 20px 40px; }
  .hero-01 h1 { font-size: clamp(32px, 8vw, 44px); }
  .hero-01 .sub { font-size: 16px; }
  .hero-01 .ctas { flex-direction: column; align-items: center; }
  .hero-01 .cta-primary, .hero-01 .cta-secondary { width: 100%; text-align: center; }
}
---HTML---
<section data-section="hero" data-variant="hero-01" class="hero-01">
  <div class="eyebrow">{{EYEBROW}}</div>
  <h1>{{HEADLINE_PART_A}} <em>{{HEADLINE_ITALIC}}</em> {{HEADLINE_PART_B}}</h1>
  <p class="sub">{{SUBHEAD}}</p>
  <div class="ctas">
    <a class="cta-primary" href="#cta">{{CTA_PRIMARY}}</a>
    <a class="cta-secondary" href="#features">{{CTA_SECONDARY}} →</a>
  </div>
  <img src="{{HERO_IMAGE_URL}}" alt="{{HERO_IMAGE_ALT}}" loading="lazy" />
</section>

---

## hero-02: split-product-shot
[category: hero]
[tags: saas, split, product]
[placeholders: EYEBROW, HEADLINE, SUBHEAD, CTA_PRIMARY, CTA_SECONDARY, TRUST_LINE, HERO_IMAGE_URL, HERO_IMAGE_ALT]
---STYLE---
.hero-02 { display: grid; grid-template-columns: 5fr 6fr; gap: 64px; align-items: center; padding: 80px 48px; max-width: 1400px; margin: 0 auto; }
.hero-02 .eyebrow { font-size: 12px; font-weight: 600; letter-spacing: 0.16em; text-transform: uppercase; color: var(--accent); margin-bottom: 16px; }
.hero-02 h1 { font-family: var(--font-display); font-size: clamp(36px, 5vw, 56px); font-weight: 500; line-height: 1.1; letter-spacing: -0.02em; color: var(--ink); margin: 0 0 20px; }
.hero-02 .sub { font-size: 17px; line-height: 1.55; color: var(--muted); margin: 0 0 28px; max-width: 500px; }
.hero-02 .ctas { display: flex; gap: 12px; margin-bottom: 24px; }
.hero-02 .cta-primary { background: var(--accent); color: var(--paper); padding: 12px 22px; border-radius: 6px; text-decoration: none; font-size: 14px; font-weight: 500; }
.hero-02 .cta-secondary { background: transparent; color: var(--ink); padding: 12px 16px; border: 1px solid var(--line); border-radius: 6px; text-decoration: none; font-size: 14px; font-weight: 500; }
.hero-02 .trust { font-size: 13px; color: var(--muted); }
.hero-02 img { width: 100%; border-radius: 12px; border: 1px solid var(--line); box-shadow: 0 12px 40px rgba(0,0,0,0.08); }
@media (max-width: 900px) { .hero-02 { grid-template-columns: 1fr; gap: 32px; padding: 48px 24px; } }
@media (max-width: 640px) {
  .hero-02 { padding: 40px 20px; }
  .hero-02 h1 { font-size: clamp(28px, 8vw, 40px); }
  .hero-02 .sub { font-size: 15px; }
  .hero-02 .ctas { flex-direction: column; }
  .hero-02 .cta-primary, .hero-02 .cta-secondary { width: 100%; text-align: center; }
}
---HTML---
<section data-section="hero" data-variant="hero-02" class="hero-02">
  <div>
    <div class="eyebrow">{{EYEBROW}}</div>
    <h1>{{HEADLINE}}</h1>
    <p class="sub">{{SUBHEAD}}</p>
    <div class="ctas">
      <a class="cta-primary" href="#cta">{{CTA_PRIMARY}}</a>
      <a class="cta-secondary" href="#features">{{CTA_SECONDARY}}</a>
    </div>
    <div class="trust">{{TRUST_LINE}}</div>
  </div>
  <img src="{{HERO_IMAGE_URL}}" alt="{{HERO_IMAGE_ALT}}" loading="lazy" />
</section>

---

## hero-03: bento-with-stats
[category: hero]
[tags: bento, dashboard, data]
[placeholders: EYEBROW, HEADLINE, SUBHEAD, CTA_PRIMARY, STAT_1_VALUE, STAT_1_LABEL, STAT_2_VALUE, STAT_2_LABEL, HERO_IMAGE_URL, HERO_IMAGE_ALT]
---STYLE---
.hero-03 { padding: 64px 32px; max-width: 1280px; margin: 0 auto; }
.hero-03 .grid { display: grid; grid-template-columns: repeat(4, 1fr); grid-template-rows: auto auto; gap: 16px; }
.hero-03 .headline-cell { grid-column: span 2; grid-row: span 2; padding: 32px; background: var(--surface); border-radius: 12px; display: flex; flex-direction: column; justify-content: center; }
.hero-03 .eyebrow { font-size: 11px; font-weight: 600; letter-spacing: 0.18em; text-transform: uppercase; color: var(--accent); margin-bottom: 16px; }
.hero-03 h1 { font-family: var(--font-display); font-size: 44px; font-weight: 500; line-height: 1.1; color: var(--ink); margin: 0 0 16px; }
.hero-03 .sub { color: var(--muted); font-size: 16px; line-height: 1.5; margin: 0 0 24px; }
.hero-03 .cta { align-self: flex-start; background: var(--ink); color: var(--paper); padding: 10px 20px; border-radius: 6px; text-decoration: none; font-size: 14px; font-weight: 500; }
.hero-03 .stat-cell { padding: 24px; background: var(--paper); border: 1px solid var(--line); border-radius: 12px; display: flex; flex-direction: column; justify-content: space-between; min-height: 140px; }
.hero-03 .stat-value { font-family: var(--font-display); font-size: 42px; font-weight: 500; color: var(--accent); }
.hero-03 .stat-label { font-size: 13px; color: var(--muted); }
.hero-03 .image-cell { grid-column: span 2; padding: 0; overflow: hidden; border-radius: 12px; border: 1px solid var(--line); }
.hero-03 .image-cell img { width: 100%; height: 100%; object-fit: cover; display: block; min-height: 200px; }
@media (max-width: 900px) { .hero-03 .grid { grid-template-columns: 1fr; } .hero-03 .headline-cell, .hero-03 .image-cell, .hero-03 .stat-cell { grid-column: 1; grid-row: auto; } .hero-03 .headline-cell { padding: 24px; } .hero-03 h1 { font-size: 32px; } }
@media (max-width: 640px) { .hero-03 { padding: 32px 16px; } .hero-03 .grid { gap: 12px; } }
---HTML---
<section data-section="hero" data-variant="hero-03" class="hero-03">
  <div class="grid">
    <div class="headline-cell">
      <div class="eyebrow">{{EYEBROW}}</div>
      <h1>{{HEADLINE}}</h1>
      <p class="sub">{{SUBHEAD}}</p>
      <a class="cta" href="#cta">{{CTA_PRIMARY}}</a>
    </div>
    <div class="stat-cell">
      <div class="stat-value">{{STAT_1_VALUE}}</div>
      <div class="stat-label">{{STAT_1_LABEL}}</div>
    </div>
    <div class="stat-cell">
      <div class="stat-value">{{STAT_2_VALUE}}</div>
      <div class="stat-label">{{STAT_2_LABEL}}</div>
    </div>
    <div class="image-cell">
      <img src="{{HERO_IMAGE_URL}}" alt="{{HERO_IMAGE_ALT}}" loading="lazy" />
    </div>
  </div>
</section>

---

## features-01: alternating-rows
[category: features]
[tags: editorial, deep, narrative]
[placeholders: SECTION_EYEBROW, SECTION_HEADLINE, FEATURE_1_TITLE, FEATURE_1_BODY, FEATURE_1_IMAGE, FEATURE_2_TITLE, FEATURE_2_BODY, FEATURE_2_IMAGE, FEATURE_3_TITLE, FEATURE_3_BODY, FEATURE_3_IMAGE]
---STYLE---
.features-01 { padding: 96px 48px; max-width: 1200px; margin: 0 auto; }
.features-01 .head { text-align: center; margin-bottom: 80px; }
.features-01 .eyebrow { font-size: 12px; font-weight: 600; letter-spacing: 0.18em; text-transform: uppercase; color: var(--accent); margin-bottom: 12px; }
.features-01 .head h2 { font-family: var(--font-display); font-size: 40px; font-weight: 500; color: var(--ink); margin: 0; max-width: 600px; margin-left: auto; margin-right: auto; line-height: 1.15; }
.features-01 .row { display: grid; grid-template-columns: 1fr 1fr; gap: 64px; align-items: center; margin-bottom: 96px; }
.features-01 .row:nth-child(even) .text { order: 2; }
.features-01 .row:nth-child(even) .image { order: 1; }
.features-01 .row h3 { font-family: var(--font-display); font-size: 32px; font-weight: 500; color: var(--ink); margin: 0 0 16px; line-height: 1.15; }
.features-01 .row p { font-size: 16px; line-height: 1.6; color: var(--ink-2); margin: 0; max-width: 460px; }
.features-01 .image img { width: 100%; border-radius: 8px; border: 1px solid var(--line); }
@media (max-width: 900px) { .features-01 .row { grid-template-columns: 1fr; gap: 24px; margin-bottom: 56px; } .features-01 .row:nth-child(even) .text, .features-01 .row:nth-child(even) .image { order: unset; } }
@media (max-width: 640px) { .features-01 { padding: 48px 20px; } .features-01 .head { margin-bottom: 40px; } .features-01 .head h2 { font-size: 28px; } .features-01 .row h3 { font-size: 24px; } }
---HTML---
<section data-section="features" data-variant="features-01" class="features-01" id="features">
  <div class="head">
    <div class="eyebrow">{{SECTION_EYEBROW}}</div>
    <h2>{{SECTION_HEADLINE}}</h2>
  </div>
  <div class="row">
    <div class="text"><h3>{{FEATURE_1_TITLE}}</h3><p>{{FEATURE_1_BODY}}</p></div>
    <div class="image"><img src="{{FEATURE_1_IMAGE}}" alt="{{FEATURE_1_TITLE}}" loading="lazy" /></div>
  </div>
  <div class="row">
    <div class="text"><h3>{{FEATURE_2_TITLE}}</h3><p>{{FEATURE_2_BODY}}</p></div>
    <div class="image"><img src="{{FEATURE_2_IMAGE}}" alt="{{FEATURE_2_TITLE}}" loading="lazy" /></div>
  </div>
  <div class="row">
    <div class="text"><h3>{{FEATURE_3_TITLE}}</h3><p>{{FEATURE_3_BODY}}</p></div>
    <div class="image"><img src="{{FEATURE_3_IMAGE}}" alt="{{FEATURE_3_TITLE}}" loading="lazy" /></div>
  </div>
</section>

---

## features-02: bento-varied
[category: features]
[tags: bento, dashboard, dense]
[placeholders: SECTION_EYEBROW, SECTION_HEADLINE, HERO_FEATURE_TITLE, HERO_FEATURE_BODY, HERO_FEATURE_IMAGE, STAT_VALUE, STAT_LABEL, QUOTE_TEXT, QUOTE_ATTR, SMALL_FEATURE_1_TITLE, SMALL_FEATURE_1_BODY, SMALL_FEATURE_2_TITLE, SMALL_FEATURE_2_BODY]
---STYLE---
.features-02 { padding: 80px 32px; max-width: 1280px; margin: 0 auto; }
.features-02 .head { margin-bottom: 40px; }
.features-02 .eyebrow { font-size: 11px; font-weight: 600; letter-spacing: 0.18em; text-transform: uppercase; color: var(--accent); margin-bottom: 8px; }
.features-02 h2 { font-family: var(--font-display); font-size: 40px; font-weight: 500; color: var(--ink); margin: 0; line-height: 1.1; max-width: 600px; }
.features-02 .grid { display: grid; grid-template-columns: repeat(4, 1fr); grid-template-rows: repeat(2, minmax(200px, auto)); gap: 16px; }
.features-02 .cell { background: var(--paper); border: 1px solid var(--line); border-radius: 12px; padding: 24px; display: flex; flex-direction: column; justify-content: space-between; }
.features-02 .hero-feature { grid-column: span 2; grid-row: span 2; padding: 32px; background: var(--surface); }
.features-02 .hero-feature h3 { font-family: var(--font-display); font-size: 28px; font-weight: 500; color: var(--ink); margin: 0 0 12px; line-height: 1.15; }
.features-02 .hero-feature p { color: var(--ink-2); font-size: 15px; line-height: 1.5; margin: 0 0 20px; }
.features-02 .hero-feature img { width: 100%; border-radius: 8px; border: 1px solid var(--line); }
.features-02 .stat { grid-column: span 1; grid-row: span 1; background: var(--paper); }
.features-02 .stat .val { font-family: var(--font-display); font-size: 48px; font-weight: 500; color: var(--accent); line-height: 1; margin-bottom: 4px; }
.features-02 .stat .lbl { font-size: 13px; color: var(--muted); }
.features-02 .quote { grid-column: span 1; grid-row: span 1; }
.features-02 .quote blockquote { font-family: var(--font-display); font-size: 18px; font-style: italic; color: var(--ink); line-height: 1.35; margin: 0 0 8px; }
.features-02 .quote cite { font-size: 12px; color: var(--muted); font-style: normal; }
.features-02 .small-feature h4 { font-size: 15px; font-weight: 600; color: var(--ink); margin: 0 0 6px; }
.features-02 .small-feature p { font-size: 13px; color: var(--muted); line-height: 1.45; margin: 0; }
@media (max-width: 900px) { .features-02 .grid { grid-template-columns: 1fr; grid-template-rows: auto; } .features-02 .hero-feature { grid-column: 1; grid-row: auto; } .features-02 .stat, .features-02 .quote, .features-02 .cell { grid-column: 1; } }
@media (max-width: 640px) { .features-02 { padding: 48px 16px; } .features-02 h2 { font-size: 28px; } .features-02 .hero-feature { padding: 24px; } .features-02 .hero-feature h3 { font-size: 22px; } }
---HTML---
<section data-section="features" data-variant="features-02" class="features-02" id="features">
  <div class="head">
    <div class="eyebrow">{{SECTION_EYEBROW}}</div>
    <h2>{{SECTION_HEADLINE}}</h2>
  </div>
  <div class="grid">
    <div class="cell hero-feature">
      <div>
        <h3>{{HERO_FEATURE_TITLE}}</h3>
        <p>{{HERO_FEATURE_BODY}}</p>
      </div>
      <img src="{{HERO_FEATURE_IMAGE}}" alt="{{HERO_FEATURE_TITLE}}" loading="lazy" />
    </div>
    <div class="cell stat">
      <div class="val">{{STAT_VALUE}}</div>
      <div class="lbl">{{STAT_LABEL}}</div>
    </div>
    <div class="cell quote">
      <blockquote>"{{QUOTE_TEXT}}"</blockquote>
      <cite>— {{QUOTE_ATTR}}</cite>
    </div>
    <div class="cell small-feature">
      <h4>{{SMALL_FEATURE_1_TITLE}}</h4>
      <p>{{SMALL_FEATURE_1_BODY}}</p>
    </div>
    <div class="cell small-feature">
      <h4>{{SMALL_FEATURE_2_TITLE}}</h4>
      <p>{{SMALL_FEATURE_2_BODY}}</p>
    </div>
  </div>
</section>

---

## features-03: icon-triad
[category: features]
[tags: saas, standard, cards]
[placeholders: SECTION_EYEBROW, SECTION_HEADLINE, SECTION_SUBHEAD, F1_TITLE, F1_BODY, F2_TITLE, F2_BODY, F3_TITLE, F3_BODY]
---STYLE---
.features-03 { padding: 96px 48px; max-width: 1200px; margin: 0 auto; }
.features-03 .head { text-align: center; margin-bottom: 64px; }
.features-03 .eyebrow { font-size: 12px; font-weight: 600; letter-spacing: 0.16em; text-transform: uppercase; color: var(--accent); margin-bottom: 12px; }
.features-03 h2 { font-family: var(--font-display); font-size: 40px; font-weight: 500; color: var(--ink); margin: 0 0 16px; line-height: 1.15; }
.features-03 .subhead { font-size: 17px; color: var(--muted); max-width: 560px; margin: 0 auto; line-height: 1.55; }
.features-03 .grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 32px; }
.features-03 .card { padding: 32px; background: var(--paper); border: 1px solid var(--line); border-radius: 12px; }
.features-03 .card .idx { font-family: var(--font-body); font-family: monospace; font-size: 12px; color: var(--accent); font-weight: 600; letter-spacing: 0.16em; margin-bottom: 20px; }
.features-03 .card h3 { font-family: var(--font-display); font-size: 22px; font-weight: 500; color: var(--ink); margin: 0 0 12px; }
.features-03 .card p { font-size: 15px; color: var(--muted); line-height: 1.55; margin: 0; }
@media (max-width: 900px) { .features-03 .grid { grid-template-columns: 1fr; gap: 20px; } }
@media (max-width: 640px) { .features-03 { padding: 48px 20px; } .features-03 h2 { font-size: 28px; } .features-03 .subhead { font-size: 15px; } }
---HTML---
<section data-section="features" data-variant="features-03" class="features-03" id="features">
  <div class="head">
    <div class="eyebrow">{{SECTION_EYEBROW}}</div>
    <h2>{{SECTION_HEADLINE}}</h2>
    <p class="subhead">{{SECTION_SUBHEAD}}</p>
  </div>
  <div class="grid">
    <div class="card"><div class="idx">01</div><h3>{{F1_TITLE}}</h3><p>{{F1_BODY}}</p></div>
    <div class="card"><div class="idx">02</div><h3>{{F2_TITLE}}</h3><p>{{F2_BODY}}</p></div>
    <div class="card"><div class="idx">03</div><h3>{{F3_TITLE}}</h3><p>{{F3_BODY}}</p></div>
  </div>
</section>

---

## testimonials-01: three-card-grid
[category: testimonials]
[tags: standard, marketing]
[placeholders: SECTION_EYEBROW, SECTION_HEADLINE, T1_QUOTE, T1_NAME, T1_ROLE, T1_AVATAR, T2_QUOTE, T2_NAME, T2_ROLE, T2_AVATAR, T3_QUOTE, T3_NAME, T3_ROLE, T3_AVATAR]
---STYLE---
.testimonials-01 { padding: 96px 48px; max-width: 1200px; margin: 0 auto; }
.testimonials-01 .head { text-align: center; margin-bottom: 48px; }
.testimonials-01 .eyebrow { font-size: 12px; font-weight: 600; letter-spacing: 0.16em; text-transform: uppercase; color: var(--accent); margin-bottom: 12px; }
.testimonials-01 h2 { font-family: var(--font-display); font-size: 36px; font-weight: 500; color: var(--ink); margin: 0; line-height: 1.15; }
.testimonials-01 .grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 24px; }
.testimonials-01 .card { padding: 28px; background: var(--paper); border: 1px solid var(--line); border-radius: 12px; }
.testimonials-01 .card blockquote { font-size: 16px; line-height: 1.55; color: var(--ink); margin: 0 0 24px; font-style: italic; }
.testimonials-01 .card .attr { display: flex; align-items: center; gap: 12px; }
.testimonials-01 .card img { width: 40px; height: 40px; border-radius: 50%; object-fit: cover; }
.testimonials-01 .card .name { font-size: 14px; font-weight: 600; color: var(--ink); }
.testimonials-01 .card .role { font-size: 12px; color: var(--muted); }
@media (max-width: 900px) { .testimonials-01 .grid { grid-template-columns: 1fr; gap: 16px; } }
@media (max-width: 640px) { .testimonials-01 { padding: 48px 20px; } .testimonials-01 h2 { font-size: 26px; } }
---HTML---
<section data-section="testimonials" data-variant="testimonials-01" class="testimonials-01">
  <div class="head">
    <div class="eyebrow">{{SECTION_EYEBROW}}</div>
    <h2>{{SECTION_HEADLINE}}</h2>
  </div>
  <div class="grid">
    <div class="card">
      <blockquote>"{{T1_QUOTE}}"</blockquote>
      <div class="attr"><img src="{{T1_AVATAR}}" alt="{{T1_NAME}}" /><div><div class="name">{{T1_NAME}}</div><div class="role">{{T1_ROLE}}</div></div></div>
    </div>
    <div class="card">
      <blockquote>"{{T2_QUOTE}}"</blockquote>
      <div class="attr"><img src="{{T2_AVATAR}}" alt="{{T2_NAME}}" /><div><div class="name">{{T2_NAME}}</div><div class="role">{{T2_ROLE}}</div></div></div>
    </div>
    <div class="card">
      <blockquote>"{{T3_QUOTE}}"</blockquote>
      <div class="attr"><img src="{{T3_AVATAR}}" alt="{{T3_NAME}}" /><div><div class="name">{{T3_NAME}}</div><div class="role">{{T3_ROLE}}</div></div></div>
    </div>
  </div>
</section>

---

## testimonials-02: hero-quote
[category: testimonials]
[tags: editorial, single, hero]
[placeholders: SECTION_EYEBROW, HERO_QUOTE, HERO_NAME, HERO_ROLE, HERO_COMPANY, HERO_AVATAR]
---STYLE---
.testimonials-02 { padding: 128px 48px; background: var(--surface); text-align: center; }
.testimonials-02 .inner { max-width: 900px; margin: 0 auto; }
.testimonials-02 .eyebrow { font-size: 11px; font-weight: 600; letter-spacing: 0.2em; text-transform: uppercase; color: var(--accent); margin-bottom: 32px; }
.testimonials-02 blockquote { font-family: var(--font-display); font-size: clamp(28px, 4vw, 44px); font-weight: 400; font-style: italic; line-height: 1.25; color: var(--ink); margin: 0 0 48px; letter-spacing: -0.01em; }
.testimonials-02 .attr { display: flex; align-items: center; gap: 16px; justify-content: center; }
.testimonials-02 .attr img { width: 56px; height: 56px; border-radius: 50%; border: 2px solid var(--paper); box-shadow: 0 2px 8px rgba(0,0,0,0.08); object-fit: cover; }
.testimonials-02 .attr .info { text-align: left; }
.testimonials-02 .attr .name { font-size: 15px; font-weight: 600; color: var(--ink); }
.testimonials-02 .attr .role { font-size: 13px; color: var(--muted); }
@media (max-width: 640px) { .testimonials-02 { padding: 72px 20px; } .testimonials-02 blockquote { font-size: 24px; } }
---HTML---
<section data-section="testimonials" data-variant="testimonials-02" class="testimonials-02">
  <div class="inner">
    <div class="eyebrow">{{SECTION_EYEBROW}}</div>
    <blockquote>"{{HERO_QUOTE}}"</blockquote>
    <div class="attr">
      <img src="{{HERO_AVATAR}}" alt="{{HERO_NAME}}" />
      <div class="info"><div class="name">{{HERO_NAME}}</div><div class="role">{{HERO_ROLE}} · {{HERO_COMPANY}}</div></div>
    </div>
  </div>
</section>

---

## pricing-01: three-tier
[category: pricing]
[tags: standard, saas]
[placeholders: SECTION_EYEBROW, SECTION_HEADLINE, T1_NAME, T1_PRICE, T1_TAGLINE, T1_F1, T1_F2, T1_F3, T1_F4, T1_CTA, T2_NAME, T2_PRICE, T2_TAGLINE, T2_F1, T2_F2, T2_F3, T2_F4, T2_F5, T2_CTA, T3_NAME, T3_PRICE, T3_TAGLINE, T3_F1, T3_F2, T3_F3, T3_F4, T3_CTA]
---STYLE---
.pricing-01 { padding: 96px 48px; max-width: 1200px; margin: 0 auto; }
.pricing-01 .head { text-align: center; margin-bottom: 56px; }
.pricing-01 .eyebrow { font-size: 12px; font-weight: 600; letter-spacing: 0.16em; text-transform: uppercase; color: var(--accent); margin-bottom: 12px; }
.pricing-01 h2 { font-family: var(--font-display); font-size: 40px; font-weight: 500; color: var(--ink); margin: 0; line-height: 1.15; }
.pricing-01 .tiers { display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; }
.pricing-01 .tier { padding: 32px; background: var(--paper); border: 1px solid var(--line); border-radius: 12px; display: flex; flex-direction: column; }
.pricing-01 .tier.recommended { border: 2px solid var(--accent); position: relative; }
.pricing-01 .tier.recommended::before { content: "Recommended"; position: absolute; top: -12px; left: 50%; transform: translateX(-50%); background: var(--accent); color: var(--paper); padding: 4px 12px; border-radius: 4px; font-size: 11px; font-weight: 600; letter-spacing: 0.08em; text-transform: uppercase; }
.pricing-01 .tier h3 { font-family: var(--font-display); font-size: 22px; font-weight: 500; color: var(--ink); margin: 0 0 12px; }
.pricing-01 .tier .price { font-family: var(--font-display); font-size: 40px; font-weight: 500; color: var(--ink); margin: 0 0 8px; }
.pricing-01 .tier .tagline { font-size: 14px; color: var(--muted); margin: 0 0 24px; line-height: 1.4; }
.pricing-01 .tier ul { list-style: none; margin: 0 0 24px; padding: 0; flex: 1; }
.pricing-01 .tier ul li { padding: 8px 0; font-size: 14px; color: var(--ink-2); border-bottom: 1px solid var(--line); }
.pricing-01 .tier ul li:last-child { border-bottom: none; }
.pricing-01 .tier .cta { display: block; text-align: center; padding: 12px; background: var(--paper); border: 1px solid var(--line); color: var(--ink); text-decoration: none; font-size: 14px; font-weight: 500; border-radius: 6px; }
.pricing-01 .tier.recommended .cta { background: var(--accent); border-color: var(--accent); color: var(--paper); }
@media (max-width: 900px) { .pricing-01 .tiers { grid-template-columns: 1fr; gap: 32px; } .pricing-01 .tier.recommended::before { top: -14px; } }
@media (max-width: 640px) { .pricing-01 { padding: 56px 20px; } .pricing-01 h2 { font-size: 28px; } .pricing-01 .tier { padding: 24px; } .pricing-01 .tier .price { font-size: 36px; } }
---HTML---
<section data-section="pricing" data-variant="pricing-01" class="pricing-01" id="pricing">
  <div class="head">
    <div class="eyebrow">{{SECTION_EYEBROW}}</div>
    <h2>{{SECTION_HEADLINE}}</h2>
  </div>
  <div class="tiers">
    <div class="tier">
      <h3>{{T1_NAME}}</h3><div class="price">{{T1_PRICE}}</div><p class="tagline">{{T1_TAGLINE}}</p>
      <ul><li>{{T1_F1}}</li><li>{{T1_F2}}</li><li>{{T1_F3}}</li><li>{{T1_F4}}</li></ul>
      <a class="cta" href="#cta">{{T1_CTA}}</a>
    </div>
    <div class="tier recommended">
      <h3>{{T2_NAME}}</h3><div class="price">{{T2_PRICE}}</div><p class="tagline">{{T2_TAGLINE}}</p>
      <ul><li>{{T2_F1}}</li><li>{{T2_F2}}</li><li>{{T2_F3}}</li><li>{{T2_F4}}</li><li>{{T2_F5}}</li></ul>
      <a class="cta" href="#cta">{{T2_CTA}}</a>
    </div>
    <div class="tier">
      <h3>{{T3_NAME}}</h3><div class="price">{{T3_PRICE}}</div><p class="tagline">{{T3_TAGLINE}}</p>
      <ul><li>{{T3_F1}}</li><li>{{T3_F2}}</li><li>{{T3_F3}}</li><li>{{T3_F4}}</li></ul>
      <a class="cta" href="#cta">{{T3_CTA}}</a>
    </div>
  </div>
</section>

---

## pricing-02: single-plan
[category: pricing]
[tags: minimal, single, focused]
[placeholders: SECTION_EYEBROW, PLAN_NAME, PLAN_PRICE, PLAN_TAGLINE, F1, F2, F3, F4, F5, F6, CTA_LABEL, GUARANTEE_LINE]
---STYLE---
.pricing-02 { padding: 96px 48px; background: var(--surface); }
.pricing-02 .inner { max-width: 620px; margin: 0 auto; text-align: center; }
.pricing-02 .eyebrow { font-size: 12px; font-weight: 600; letter-spacing: 0.18em; text-transform: uppercase; color: var(--accent); margin-bottom: 16px; }
.pricing-02 h3 { font-family: var(--font-display); font-size: 32px; font-weight: 500; color: var(--ink); margin: 0 0 8px; }
.pricing-02 .price { font-family: var(--font-display); font-size: 72px; font-weight: 500; color: var(--accent); margin: 0 0 8px; line-height: 1; }
.pricing-02 .tagline { font-size: 16px; color: var(--muted); margin: 0 0 40px; }
.pricing-02 .features { list-style: none; margin: 0 0 32px; padding: 0; text-align: left; background: var(--paper); border: 1px solid var(--line); border-radius: 12px; padding: 24px 32px; }
.pricing-02 .features li { padding: 10px 0; font-size: 15px; color: var(--ink-2); border-bottom: 1px solid var(--line); }
.pricing-02 .features li:last-child { border-bottom: none; }
.pricing-02 .features li::before { content: "→"; color: var(--accent); margin-right: 12px; font-weight: 500; }
.pricing-02 .cta { display: inline-block; background: var(--ink); color: var(--paper); padding: 16px 32px; border-radius: 6px; text-decoration: none; font-size: 15px; font-weight: 500; margin-bottom: 16px; }
.pricing-02 .guarantee { font-size: 13px; color: var(--muted); font-style: italic; }
@media (max-width: 640px) { .pricing-02 { padding: 56px 20px; } .pricing-02 .price { font-size: 56px; } .pricing-02 .features { padding: 20px 24px; } }
---HTML---
<section data-section="pricing" data-variant="pricing-02" class="pricing-02" id="pricing">
  <div class="inner">
    <div class="eyebrow">{{SECTION_EYEBROW}}</div>
    <h3>{{PLAN_NAME}}</h3>
    <div class="price">{{PLAN_PRICE}}</div>
    <p class="tagline">{{PLAN_TAGLINE}}</p>
    <ul class="features">
      <li>{{F1}}</li><li>{{F2}}</li><li>{{F3}}</li>
      <li>{{F4}}</li><li>{{F5}}</li><li>{{F6}}</li>
    </ul>
    <a class="cta" href="#cta">{{CTA_LABEL}}</a>
    <div class="guarantee">{{GUARANTEE_LINE}}</div>
  </div>
</section>

---

## cta-01: centered-band
[category: cta]
[tags: classic, marketing]
[placeholders: EYEBROW, HEADLINE, SUBHEAD, CTA_PRIMARY, CTA_SECONDARY]
---STYLE---
.cta-01 { padding: 96px 48px; background: var(--ink); text-align: center; }
.cta-01 .inner { max-width: 720px; margin: 0 auto; }
.cta-01 .eyebrow { font-size: 12px; font-weight: 600; letter-spacing: 0.18em; text-transform: uppercase; color: var(--accent-2); margin-bottom: 16px; }
.cta-01 h2 { font-family: var(--font-display); font-size: clamp(32px, 4vw, 48px); font-weight: 500; color: var(--paper); margin: 0 0 20px; line-height: 1.15; }
.cta-01 .sub { font-size: 18px; color: rgba(255,255,255,0.7); margin: 0 0 32px; line-height: 1.5; }
.cta-01 .ctas { display: flex; gap: 12px; justify-content: center; }
.cta-01 .primary { background: var(--accent); color: var(--paper); padding: 14px 28px; border-radius: 6px; text-decoration: none; font-size: 15px; font-weight: 500; }
.cta-01 .secondary { color: var(--paper); padding: 14px 20px; text-decoration: none; font-size: 15px; font-weight: 500; border: 1px solid rgba(255,255,255,0.2); border-radius: 6px; }
@media (max-width: 640px) { .cta-01 { padding: 64px 20px; } .cta-01 h2 { font-size: 28px; } .cta-01 .sub { font-size: 16px; } .cta-01 .ctas { flex-direction: column; } .cta-01 .primary, .cta-01 .secondary { width: 100%; text-align: center; } }
---HTML---
<section data-section="cta" data-variant="cta-01" class="cta-01" id="cta">
  <div class="inner">
    <div class="eyebrow">{{EYEBROW}}</div>
    <h2>{{HEADLINE}}</h2>
    <p class="sub">{{SUBHEAD}}</p>
    <div class="ctas">
      <a class="primary" href="#">{{CTA_PRIMARY}}</a>
      <a class="secondary" href="#">{{CTA_SECONDARY}}</a>
    </div>
  </div>
</section>

---

## cta-02: split-with-form
[category: cta]
[tags: form, capture, engagement]
[placeholders: HEADLINE, SUBHEAD, INPUT_PLACEHOLDER, CTA_LABEL, TRUST_LINE]
---STYLE---
.cta-02 { padding: 96px 48px; max-width: 1200px; margin: 0 auto; }
.cta-02 .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 64px; align-items: center; padding: 48px; background: var(--surface); border-radius: 16px; }
.cta-02 h2 { font-family: var(--font-display); font-size: 36px; font-weight: 500; color: var(--ink); margin: 0 0 16px; line-height: 1.15; }
.cta-02 .sub { color: var(--muted); font-size: 16px; line-height: 1.55; margin: 0; }
.cta-02 .form { display: flex; gap: 8px; margin-bottom: 12px; }
.cta-02 input { flex: 1; padding: 14px 16px; background: var(--paper); border: 1px solid var(--line); border-radius: 6px; font-size: 15px; color: var(--ink); font-family: inherit; outline: none; }
.cta-02 input:focus { border-color: var(--accent); }
.cta-02 .btn { background: var(--ink); color: var(--paper); padding: 14px 20px; border: none; border-radius: 6px; font-size: 14px; font-weight: 500; cursor: pointer; font-family: inherit; }
.cta-02 .trust { font-size: 12px; color: var(--muted); }
@media (max-width: 900px) { .cta-02 .grid { grid-template-columns: 1fr; padding: 32px; gap: 24px; } }
@media (max-width: 640px) { .cta-02 { padding: 48px 16px; } .cta-02 h2 { font-size: 26px; } .cta-02 .form { flex-direction: column; } .cta-02 input, .cta-02 .btn { width: 100%; } }
---HTML---
<section data-section="cta" data-variant="cta-02" class="cta-02" id="cta">
  <div class="grid">
    <div>
      <h2>{{HEADLINE}}</h2>
      <p class="sub">{{SUBHEAD}}</p>
    </div>
    <div>
      <form class="form" onsubmit="event.preventDefault()">
        <input type="email" placeholder="{{INPUT_PLACEHOLDER}}" />
        <button type="submit" class="btn">{{CTA_LABEL}}</button>
      </form>
      <div class="trust">{{TRUST_LINE}}</div>
    </div>
  </div>
</section>

---

## footer-01: minimal-hairline
[category: footer]
[tags: minimal, editorial]
[placeholders: BRAND_NAME, TAGLINE, C1_TITLE, C1_L1, C1_L2, C1_L3, C2_TITLE, C2_L1, C2_L2, C2_L3, C3_TITLE, C3_L1, C3_L2, C3_L3, COPYRIGHT, ATTRIBUTION]
---STYLE---
.footer-01 { padding: 64px 48px 32px; border-top: 1px solid var(--line); background: var(--paper); }
.footer-01 .inner { max-width: 1200px; margin: 0 auto; }
.footer-01 .top { display: grid; grid-template-columns: 2fr 1fr 1fr 1fr; gap: 48px; margin-bottom: 48px; }
.footer-01 .brand { font-family: var(--font-display); font-size: 24px; color: var(--ink); margin-bottom: 12px; }
.footer-01 .tagline { font-size: 14px; color: var(--muted); line-height: 1.5; max-width: 280px; }
.footer-01 .col h4 { font-size: 12px; font-weight: 600; letter-spacing: 0.14em; text-transform: uppercase; color: var(--muted); margin: 0 0 16px; }
.footer-01 .col ul { list-style: none; margin: 0; padding: 0; }
.footer-01 .col ul li { margin-bottom: 8px; }
.footer-01 .col ul li a { font-size: 14px; color: var(--ink-2); text-decoration: none; }
.footer-01 .col ul li a:hover { color: var(--accent); }
.footer-01 .bottom { padding-top: 24px; border-top: 1px solid var(--line); display: flex; justify-content: space-between; align-items: center; font-size: 12px; color: var(--muted); }
@media (max-width: 900px) { .footer-01 .top { grid-template-columns: 1fr 1fr; gap: 32px; } }
@media (max-width: 640px) { .footer-01 { padding: 48px 20px 24px; } .footer-01 .top { grid-template-columns: 1fr; gap: 28px; margin-bottom: 32px; } .footer-01 .bottom { flex-direction: column; align-items: flex-start; gap: 8px; } }
---HTML---
<footer data-section="footer" data-variant="footer-01" class="footer-01">
  <div class="inner">
    <div class="top">
      <div><div class="brand">{{BRAND_NAME}}</div><div class="tagline">{{TAGLINE}}</div></div>
      <div class="col"><h4>{{C1_TITLE}}</h4><ul><li><a href="#">{{C1_L1}}</a></li><li><a href="#">{{C1_L2}}</a></li><li><a href="#">{{C1_L3}}</a></li></ul></div>
      <div class="col"><h4>{{C2_TITLE}}</h4><ul><li><a href="#">{{C2_L1}}</a></li><li><a href="#">{{C2_L2}}</a></li><li><a href="#">{{C2_L3}}</a></li></ul></div>
      <div class="col"><h4>{{C3_TITLE}}</h4><ul><li><a href="#">{{C3_L1}}</a></li><li><a href="#">{{C3_L2}}</a></li><li><a href="#">{{C3_L3}}</a></li></ul></div>
    </div>
    <div class="bottom"><span>{{COPYRIGHT}}</span><span>{{ATTRIBUTION}}</span></div>
  </div>
</footer>

---

## footer-02: giant-wordmark
[category: footer]
[tags: bold, editorial, signature]
[placeholders: BRAND_NAME, TAGLINE, LINK_1, LINK_2, LINK_3, LINK_4, LINK_5, LINK_6, ATTRIBUTION]
---STYLE---
.footer-02 { padding: 96px 48px 32px; background: var(--ink); color: var(--paper); }
.footer-02 .inner { max-width: 1400px; margin: 0 auto; }
.footer-02 .wordmark { font-family: var(--font-display); font-size: clamp(80px, 15vw, 200px); font-weight: 400; letter-spacing: -0.04em; line-height: 0.9; color: var(--paper); margin: 0 0 48px; }
.footer-02 .row { display: grid; grid-template-columns: 2fr 3fr; gap: 48px; padding-top: 32px; border-top: 1px solid rgba(255,255,255,0.15); }
.footer-02 .tagline { font-size: 15px; color: rgba(255,255,255,0.7); max-width: 320px; line-height: 1.5; }
.footer-02 .links { display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; }
.footer-02 .links a { color: rgba(255,255,255,0.75); text-decoration: none; font-size: 14px; padding: 4px 0; }
.footer-02 .links a:hover { color: var(--accent-2); }
.footer-02 .attribution { padding-top: 24px; margin-top: 32px; border-top: 1px solid rgba(255,255,255,0.1); font-size: 12px; color: rgba(255,255,255,0.4); }
@media (max-width: 900px) { .footer-02 .row { grid-template-columns: 1fr; gap: 24px; } .footer-02 .links { grid-template-columns: repeat(2, 1fr); } }
@media (max-width: 640px) { .footer-02 { padding: 64px 20px 24px; } .footer-02 .wordmark { font-size: clamp(56px, 20vw, 96px); } .footer-02 .links { grid-template-columns: 1fr; } }
---HTML---
<footer data-section="footer" data-variant="footer-02" class="footer-02">
  <div class="inner">
    <div class="wordmark">{{BRAND_NAME}}</div>
    <div class="row">
      <div class="tagline">{{TAGLINE}}</div>
      <div class="links">
        <a href="#">{{LINK_1}}</a><a href="#">{{LINK_2}}</a><a href="#">{{LINK_3}}</a>
        <a href="#">{{LINK_4}}</a><a href="#">{{LINK_5}}</a><a href="#">{{LINK_6}}</a>
      </div>
    </div>
    <div class="attribution">{{ATTRIBUTION}}</div>
  </div>
</footer>
