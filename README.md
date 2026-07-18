# lucid-design

A native macOS app for generating high-quality single-file HTML designs. Pick sections from a hand-authored variant library, let the LLM fill the copy, swap sections zero-cost after the design lands.

Not a Figma clone. Not a template gallery. A **design kit configurator** — you choose nav / hero / features / testimonials / pricing / CTA / footer variants from a curated catalog, plus a palette and theme, and the assembler stitches them into one clean HTML file with real copy written by the LLM.

Built in Rust (`tao` + `wry`) with a single-file HTML frontend rendered in WKWebView. One binary, no web server, no Electron.

---

## Install

```bash
cargo install design-gen
```

First install compiles from source and takes ~2 minutes. Requires Xcode Command Line Tools.

Or clone and build locally:

```bash
git clone https://github.com/ankush109/lucid-design
cd lucid-design
cargo build --release
./target/release/design-gen
```

> **macOS only.** Requires macOS 11+ and Xcode CLI tools (`xcode-select --install`).

---

## Setup

On first launch the app looks for `config.toml` in the current directory. Create one:

```toml
provider = "claudecode"   # uses your local Claude Code CLI — no API key needed
model    = ""
api_key  = ""
```

**Other supported providers:**

| Provider | `provider` value | Notes |
|---|---|---|
| Claude Code CLI | `claudecode` | Uses the local `claude` binary. Free if you have Claude Code. |
| Anthropic API | `anthropic` | Direct API with prompt caching (90% cost savings on repeat calls). |
| OpenAI | `openai` | Set `api_key` and `model = "gpt-4o"`. |
| Groq | `groq` | Fast inference, has a free tier. |
| Ollama | `ollama` | Local models, no API key. Set `base_url = "http://localhost:11434/v1"`. |
| Gemini | `gemini` | Google Gemini API. |

---

## How it works

### 1. Describe your product

Type your idea in the chat pane:

```
a fitness tracking app landing page
```

A kit picker appears with chip rows for each section category — **Theme, Palette, Navbar, Hero, Features, Testimonials, Pricing, CTA, Footer**. Every row defaults to **Auto**. Leave everything on Auto for a fully-generated design. Pick specific variants where you have opinions.

### 2. Build

Hit Send. The assembler:

1. Looks up each picked variant in the compiled library (17 hand-authored HTML fragments)
2. Collects all placeholders — typically 40–80 unique keys like `{{HEADLINE}}`, `{{HERO_IMAGE_URL}}`, `{{T1_NAME}}`
3. Makes **one LLM call** asking for a JSON dict mapping each placeholder to a filled string (specific copy, real image URLs, named testimonials)
4. Interpolates the fills into the variant HTML

Sections stream into the canvas top-down with a brief fade-in per section so the build feels intentional.

### 3. Swap sections zero-cost

Click any section on the canvas → a **↻ Swap this section** panel shows the alternate variants for that category. Click one → Rust splices the new variant in place, preserving content via structural mapping. **Zero LLM tokens.**

### 4. Refine via chat

Type anything after the design lands → the LLM edits the current HTML. Click an element first to scope the edit to just that element.

### 5. Export

**↓ Export HTML** saves a self-contained HTML file to `~/Documents/lucid-design/`. Open it in any browser — no server, no dependencies.

---

## The variant library

17 hand-authored HTML sections in `src/ai/variants.md`, compiled into the binary at build time:

| Category | Variants |
|---|---|
| Navbar | brand-heavy-serif · sticky-transparent · centered-editorial |
| Hero | centered-editorial · split-product-shot · bento-with-stats |
| Features | alternating-rows · bento-varied · icon-triad |
| Testimonials | three-card-grid · hero-quote |
| Pricing | three-tier · single-plan |
| CTA | centered-band · split-with-form |
| Footer | minimal-hairline · giant-wordmark |

Each variant uses CSS custom properties (`var(--accent)`, `var(--paper)`, `var(--ink)`) so palette swaps re-tone the whole section without touching HTML. All variants have responsive breakpoints.

**10 palettes** (`src/ai/palettes.md`) — warm-cream-brick, minimal-white, dark-refined, corporate-blue, warm-earth, clinical-teal, fashion-mono, cyber-neon, sunset-terracotta, forest-paper.

**12 themes** (`src/ai/themes.md`) — each bundles a palette, font pairing, radius scale, and motion vocabulary. Examples: `editorial-warm-cream`, `saas-corporate-blue`, `luxury-fashion`, `brutalist-mono`, `cyber-neon-dark`.

---

## Where your work is saved

```
~/Documents/lucid-design/
├── my-app.html           ← current design HTML
├── my-app.name           ← display name
├── my-app.chat.json      ← full chat log, restored on reopen
└── ...
```

Every canvas edit, section swap, and refine autosaves. Reopen a project and the full conversation comes back.

Exported designs go to `~/Documents/lucid-design/design-export-<timestamp>.html`.

---

## Project structure

```
src/
├── main.rs               # entry: config, mac menu, wry webview, event loop
├── config.rs             # config.toml parsing
├── pipeline.rs           # IPC routing, assembly, swap, refine, critique
├── variants.rs           # library parser (OnceLock<Library>)
├── projects.rs           # ~/Documents/lucid-design/ file I/O
├── knowledge.rs          # saved design patterns
├── scraper/              # reference URL fetcher
└── ai/
    ├── mod.rs            # AiProvider trait
    ├── prompts.rs        # system prompt + prompt builders
    ├── anthropic.rs      # Anthropic API with cache_control
    ├── openai.rs         # OpenAI-compatible (Groq / Mistral / Together / Ollama)
    ├── gemini.rs
    ├── claudecode.rs     # local Claude Code CLI subprocess
    ├── design_knowledge.md   # 14 sections of UI/UX doctrine (~14k tokens)
    ├── variants.md           # 17 hand-authored HTML fragments
    ├── palettes.md           # 10 palette :root blocks
    └── themes.md             # 12 theme kits
assets/
└── ui.html               # single-file frontend (HTML + CSS + JS, no build step)
```

---

## Dependencies

```
wry 0.55 · tao 0.35 · tokio 1 · reqwest 0.12 · serde_json 1 · scraper 0.20 · anyhow 1
[macOS] cocoa 0.25 · objc 0.2
```

---

## License

MIT OR Apache-2.0
